import socket

def send_command(command):
    host = '192.168.0.175'   # Ensure this matches the IP address of the device your rust node is live on
    port = 8080  # Ensure this matches the port your node is listening on

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((host, port))
        s.sendall((command + '\n').encode())  # Ensure command ends with newline
        data = s.recv(8192)  # Adjust buffer size if needed

    return data.decode()

# Add a block to the blockchain
# add_block_response = send_command("ADD_BLOCK Your block D")
# print("Response from adding block:", add_block_response)



add_user_command = send_command("ADD_USER Nikhil 1000")
print(add_user_command)

add_user_command = send_command("ADD_USER Pranjal 1000")
print(add_user_command)

add_user_command = send_command("ADD_USER Aditya 1000")
print(add_user_command)

# Send multiple transactions
transaction_response = send_command("TRANSACTION Pranjal Nikhil 100 Nikhil Aditya 200 Aditya Pranjal 300")
print("Response from adding transactions:", transaction_response)

# Print the USer DAG
user_DAG_response = send_command("PRINT_USER_DAG Pranjal")
print("Response from adding block:", user_DAG_response)

# Print the User2 DAG
user_DAG_response = send_command("PRINT_USER_DAG Nikhil")
print("Response from adding block:", user_DAG_response)

# Print the entire blockchain
print_chain_response = send_command("PRINT_DAG")
print("Blockchain content:\n", print_chain_response)

