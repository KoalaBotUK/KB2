from dislord import interaction, server, serverless


@interaction.command(name="hello")
def hello():
    return "hello world!"


def serverless_handler(*args, **kwargs):
    return serverless.serverless_handler(*args, **kwargs)


if __name__ == '__main__':
    server.start_server()
