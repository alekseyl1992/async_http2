import asyncio
import logging

from async_http2 import Client


def main():
    logging.basicConfig(level=logging.INFO, format='%(asctime)-15s %(message)s')

    client = Client(timeout=60)

    loop = asyncio.get_event_loop()
    loop.run_until_complete(work(client))


async def work(client):
    resp_data = await client.get('http://localhost:1010', {
        'fast': '1',
    })
    print(resp_data)


if __name__ == '__main__':
    main()
