import asyncio
import logging
import pathlib
import shutil

TESTS_ROOT = pathlib.Path(__file__).parent
SOURCE_LIB = TESTS_ROOT.parent / 'target' / 'i686-pc-windows-msvc' / 'debug' / 'async_http2.dll'
TARGET_PATH = TESTS_ROOT / 'async_http2.pyd'


def main():
    logging.basicConfig(level=logging.INFO, format='%(asctime)-15s %(message)s')

    shutil.copy(SOURCE_LIB, TARGET_PATH)

    from async_http2 import Client
    client = Client(timeout=60)

    loop = asyncio.get_event_loop()
    loop.run_until_complete(work(client))


async def work(client):
    t1 = asyncio.create_task(
        t(client, 'http://localhost:1010', params={'fast': '1'}))
    t2 = asyncio.create_task(
        t(client, 'http://localhost:1010', params={}))

    await asyncio.gather(t1, t2)


async def t(client, url: str, params: dict):
    while True:
        res: str = await client.get(url, params=params)
        logging.info(res.strip())


if __name__ == '__main__':
    main()
