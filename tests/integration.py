import unittest
from socket import (socket, AF_INET, SOCK_STREAM)


RESPONSE_CHUNK_SIZE = 4096


class Proxy:
    def __init__(self, address, port):
        self.proxy = socket(AF_INET, SOCK_STREAM)
        self.proxy.connect((address, port))

    def teardown(self):
        self.proxy.close()

    def send_v1_request(self):
        request = b'tcp@216.239.38.120$80\r\nGET / HTTP/1.1\r\nHost: google.com\r\nConnection: close\r\n\r\n'
        self.proxy.sendall(request)
        response = self.proxy.recv(RESPONSE_CHUNK_SIZE)
        return response.decode('utf-8')

    def send_v2_request(self):
        request = b'\x00\x0a\x01\x00\x00\xd8\xef\x26\x78\x50\r\nGET / HTTP/1.1\r\nHost: google.com\r\nConnection: close\r\n\r\n'
        self.proxy.sendall(request)
        response = self.proxy.recv(RESPONSE_CHUNK_SIZE)
        return response.decode('utf-8')


class Test(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.proxy = Proxy('127.0.0.1', 6666)

    @classmethod
    def tearDownClass(cls):
        cls.proxy.teardown()

    def test_v1(self):
        response = self.proxy.send_v1_request()
        status = response[:30]
        self.assertEqual(status, 'HTTP/1.1 301 Moved Permanently')

    def test_v2(self):
        response = self.proxy.send_v2_request()
        status = response[:30]
        self.assertEqual(status, 'HTTP/1.1 301 Moved Permanently')


if __name__ == '__main__':
    unittest.main()
