from dislord import CommandGroup
from kb2.client import client

verify_group = CommandGroup(client=client, name="verify", description="Configure Guild Verification")
