import yandexcloud
import logging
import tomlkit
import json
import base64
import os
import ydb
import ydb.iam
import db_conn


logging.getLogger().setLevel(logging.DEBUG)

async def handler(event, context):
    data = None
    if event['isBase64Encoded']:
        data = base64.b64decode(event['body'])
    else:
        data = event['body']

    body = json.loads(data)

    logging.debug(body)

    id = body['id']
    result = body['result']

    pool = await db_conn.conn_pool()

    await pool.execute_with_retries(
        """
        DECLARE $id AS Int64;
        DECLARE $result AS Bool;

        UPDATE records
            SET result = $result
        WHERE id = $id;
        """,
        {
            "$id": id,
            "$result": result,
        }
    )

    await pool.stop()

    return {
        'statusCode': 200,
        'headers': {
            'Access-Control-Allow-Origin': '*'
        },
        'body': True,
    }
