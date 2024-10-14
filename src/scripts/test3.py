import socket

def send_command(command):
    host = '192.168.1.12'   # Ensure this matches the IP address of the device your rust node is live on
    port = 8080  # Ensure this matches the port your node is listening on

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((host, port))
        s.sendall((command + '\n').encode())  # Ensure command ends with newline
        data = s.recv(8192)  # Adjust buffer size if needed

    return data.decode()


alice_response = send_command("ADD_USER Alice 1000")
print(alice_response)

bob_response = send_command("ADD_USER Bob 500")
print(bob_response)



transaction_response = send_command("TRANSACTION TOKEN Alice Bob 200")
print(transaction_response)



valid_response = send_command("VALIDATE_LOCAL_DAG Alice")
print(valid_response)

