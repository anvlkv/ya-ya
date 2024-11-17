import logging
import tomlkit
import os
import json
import base64

from yandex_cloud_ml_sdk import AsyncYCloudML
from langchain_core.messages import AIMessage, HumanMessage, SystemMessage


with open('./prompts.toml', 'r', encoding='utf-8') as file:
    file_str = file.read()

prompts = tomlkit.parse(file_str)

logging.getLogger().setLevel(logging.DEBUG)


def maybe_previous(previous):
    if previous:
        return [
            AIMessage(content=previous),
            HumanMessage(content=prompts['user']['reword']),
        ]

    return []

def example_prompts():
    ctx_prompt = prompts['user']['word']['ctx_prompt']
    word_prompt = prompts['user']['word']['word_prompt']


    ex = []

    for v in prompts['examples']:
        ex.extend([
            HumanMessage(content="{word_prompt} <СЛОВО>{translate_word}</СЛОВО> \n\n {ctx_prompt} <КОНТЕКСТ>{translate_ctx}</КОНТЕКСТ>".format(
                ctx_prompt=ctx_prompt,
                word_prompt=word_prompt,
                translate_word = v['user']['word'],
                translate_ctx = v['user']['context']
            )),
            AIMessage(content=v['assistant']['word'])
        ])

    return ex



async def handler(event, context):
    logging.debug(prompts)

    data = None
    if event['isBase64Encoded']:
        data = base64.b64decode(event['body'])
    else:
        data = event['body']

    body = json.loads(data)

    translate_word = body['word']
    translate_ctx = body['context']
    translate_previous = body['previous']

    ctx_prompt = prompts['user']['word']['ctx_prompt']
    word_prompt = prompts['user']['word']['word_prompt']

    translate_prompt = "{word_prompt} <СЛОВО>{translate_word}</СЛОВО> \n\n {ctx_prompt} <КОНТЕКСТ>{translate_ctx}</КОНТЕКСТ>".format(
        ctx_prompt=ctx_prompt,
        word_prompt=word_prompt,
        translate_word = translate_word,
        translate_ctx = translate_ctx
    )


    iam_token = context.token['access_token']
    sdk = AsyncYCloudML(folder_id=os.environ['FN_MODEL_FOLDER_ID'], auth=iam_token)
    model = sdk.models.completions('yandexgpt-lite', model_version='rc')
    model = model.configure(temperature=0.5).langchain(model_type="chat", timeout=context.get_remaining_time_in_millis())

    langchain_result = await model.ainvoke([
        SystemMessage(content=prompts['system']['word']['goal']),
        SystemMessage(content=prompts['system']['rules']),
        SystemMessage(content=prompts['system']['word']['template']),
        SystemMessage(content=prompts['system']['word']['interlude']),
        *example_prompts(),
        HumanMessage(content=prompts['user']['word']['prompt']),
        HumanMessage(content=translate_prompt),
        *maybe_previous(translate_previous)
    ])

    logging.info("{} tokens used".format(langchain_result))

    return {
        'statusCode': 200,
        'headers': {
            'Content-Type': 'text/plain',
            'Access-Control-Allow-Origin': '*'
        },
        'isBase64Encoded': False,
        # 'body': "# Dummy"
        'body': langchain_result.content
    }
