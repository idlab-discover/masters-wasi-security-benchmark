#!/usr/bin/env python3
import asyncio

async def handle_client(reader, writer):
    try:
        while True:
            request_line = await reader.readline()
            if not request_line:
                break
                
            content_length = 0
            keep_alive = True
            
            while True:
                line = await reader.readline()
                if not line or line == b'\r\n':
                    break
                
                line_lower = line.lower()
                if line_lower.startswith(b'content-length:'):
                    content_length = int(line_lower.split(b':')[1].strip())
                elif line_lower.startswith(b'connection:'):
                    if b'close' in line_lower:
                        keep_alive = False
            
            if content_length > 0:
                await reader.readexactly(content_length)
                
            response = (
                b"HTTP/1.1 200 OK\r\n"
                b"Content-Length: 13\r\n"
                b"Content-Type: text/plain\r\n"
                b"Connection: " + (b"keep-alive" if keep_alive else b"close") + b"\r\n"
                b"\r\n"
                b"Hello, World!"
            )
            writer.write(response)
            await writer.drain()
            
            if not keep_alive:
                break
                
    except asyncio.IncompleteReadError:
        pass
    except Exception:
        pass
    finally:
        writer.close()
        try:
            await writer.wait_closed()
        except:
            pass

async def main():
    server = await asyncio.start_server(handle_client, '127.0.0.1', 3006)
    async with server:
        await server.serve_forever()

if __name__ == '__main__':
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
