import socket

def send_command(command):
    host = '192.168.1.14'   # Ensure this matches the IP address of the device your rust node is live on
    port = 8080  # Ensure this matches the port your node is listening on

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((host, port))
        s.sendall((command + '\n').encode())  # Ensure command ends with newline
        data = s.recv(8192)  # Adjust buffer size if needed

    return data.decode()


# Check balance for Alice
response = send_command("CHECK_BALANCE Nikhil")
print(response)

# # Check balance for Bob
# response = send_command("CHECK_BALANCE Bob")
# print(response)

# # Check balance for Charlie
# response = send_command("CHECK_BALANCE Charlie")
# print(response)
