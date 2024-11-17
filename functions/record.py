import yandexcloud
import logging
import tomlkit
import os
import json
import base64
import ydb
import ydb.iam


logging.getLogger().setLevel(logging.DEBUG)

driver = ydb.Driver(
    endpoint=os.environ['YDB_ENDPOINT'],
    database=os.environ['YDB_DATABASE'],
    credentials=ydb.iam.MetadataUrlCredentials(),
)

driver.wait(fail_fast=True, timeout=5)


pool = ydb.SessionPool(driver)

def store_word_record(session, word, context, translation, result):
    params = {
        "$word": (word, ydb.PrimitiveType.String),
        "$context": (context, ydb.PrimitiveType.String),
        "$translation": (translation, ydb.PrimitiveType.String),
        "$result": (result, ydb.PrimitiveType.Bool),
    }

    return session.transaction().execute(
        """
        DECLARE $word AS String;
        DECLARE $context AS String;
        DECLARE $translation AS String;
        DECLARE $result AS Bool;

        REPLACE INTO records ( word, context, translation, result )
            VALUES ( $word, $context, $translation, $result );
        """,
        parameters = params,
        commit_tx=True,
        settings=ydb.BaseRequestSettings()
    )


def handler(event, context):
    data = None
    if event['isBase64Encoded']:
        data = base64.b64decode(event['body'])
    else:
        data = event['body']

    body = json.loads(data)

    logging.debug(body)

    word = body['word']
    ctx = body['context']
    translation = body['translation']
    result = body['result']

    pool.retry_operation_sync(store_word_record, word=word, context=ctx, translation=translation, result=result)


    return {
        'statusCode': 200,
        'body': True,
    }
