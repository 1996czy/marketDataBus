
Create a market data bus to subscribe and distributing data to according handlers

Here the server example will establish connection to specific exchange and send subscription message after it received data requests from handlers. Up to 300 handlers are designed to send request to subscribe one contract depth data stream and print it out when get feedback.

Assuming the server and the handlers are running on the same machine, the fastest way to do IPC is memory sharing methods, unix socket is one simple but kinda slow implementation of them.

Also, if there is need to do IPC between different machine, this example would be really convenient to transform into tcp or udp stream base on socket layer.
