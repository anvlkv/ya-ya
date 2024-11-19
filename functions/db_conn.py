import os
import ydb
import ydb.iam

async def conn_pool():
    driver = ydb.aio.Driver(
        endpoint=os.environ['YDB_ENDPOINT'],
        database=os.environ['YDB_DATABASE'],
        credentials=ydb.iam.MetadataUrlCredentials(),
    )

    await driver.wait(fail_fast=True, timeout=5)


    pool = ydb.aio.QuerySessionPool(driver)

    return pool
