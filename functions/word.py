import logging
import tomlkit
import os
import json
import base64
import db_conn


from yandex_cloud_ml_sdk import AsyncYCloudML
from langchain_core.messages import AIMessage, HumanMessage, SystemMessage


with open('./prompts.toml', 'r', encoding='utf-8') as file:
    file_str = file.read()

prompts = tomlkit.parse(file_str)

logging.getLogger().setLevel(logging.DEBUG)

async def handler(event, context):
    logging.debug(prompts)

    data = None
    if event['isBase64Encoded']:
        data = base64.b64decode(event['body'])
    else:
        data = event['body']

    body = json.loads(data)
    origin = event['headers']['Origin']

    word = body['word']
    ctx = body['context']
    annotation_prev = body['previous']

    ctx_prompt = prompts['user']['word']['ctx_prompt']
    word_prompt = prompts['user']['word']['word_prompt']

    translate_prompt = "{word_prompt} <СЛОВО>{translate_word}</СЛОВО> \n\n {ctx_prompt} <КОНТЕКСТ>{translate_ctx}</КОНТЕКСТ>".format(
        ctx_prompt=ctx_prompt,
        word_prompt=word_prompt,
        translate_word = word,
        translate_ctx = ctx
    )


    iam_token = context.token['access_token']
    sdk = AsyncYCloudML(folder_id=os.environ['FN_MODEL_FOLDER_ID'], auth=iam_token)
    model = sdk.models.completions('yandexgpt-lite', model_version='rc')
    model = model.configure(temperature=0.42).langchain(model_type="chat", timeout=context.get_remaining_time_in_millis())

    langchain_result = await model.ainvoke([
        SystemMessage(content=prompts['system']['word']['goal']),
        SystemMessage(content=prompts['system']['rules']),
        SystemMessage(content=prompts['system']['word']['template']),
        SystemMessage(content=prompts['system']['word']['interlude']),
        *example_prompts(),
        HumanMessage(content=prompts['user']['word']['prompt']),
        HumanMessage(content=translate_prompt),
        *maybe_previous(annotation_prev)
    ])

    logging.info("{}".format(langchain_result.usage_metadata))

    pool = await db_conn.conn_pool()

    result_sets = await pool.execute_with_retries(
        """
        DECLARE $word AS Utf8;
        DECLARE $context AS Utf8;
        DECLARE $annotation AS Utf8;
        DECLARE $origin AS Utf8;

        INSERT INTO records (word, context, annotation, origin)
            VALUES($word, $context, $annotation, $origin) RETURNING id;
        """,
        {
            "$word": word,
            "$context": ctx,
            "$annotation": langchain_result.content,
            "$origin": origin,
        }
    )

    id = result_sets[0].rows[0].id

    await pool.stop()

    return {
        'statusCode': 200,
        'headers': {
            'Content-Type': 'application/json',
            'Access-Control-Allow-Origin': origin
        },
        'body': {
            'id': id,
            'annotation': langchain_result.content,
        }
    }


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
