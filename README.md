# async-http2
[![GitHub Actions](https://github.com/alekseyl1992/async_http2/workflows/Python/badge.svg)](https://github.com/alekseyl1992/async_http2/actions?query=workflow%3APython)
[![PyPI](https://img.shields.io/pypi/v/async-http2.svg)](https://pypi.org/project/async-http2)

asyncio-compatible HTTP2 client for Python based on `reqwest` Rust crate and pyo3/pyo3-asyncio

## Installation

```bash
pip install async-http2
```

## Usage

```python3
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
    logging.info(resp_data)


if __name__ == '__main__':
    main()
```

## License

This work is released under the MIT license. A copy of the license is provided in the [LICENSE](../LICENSE) file.
