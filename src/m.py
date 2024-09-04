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
add_block_response = send_command("ADD_BLOCK Your block D2")
print("Response from adding block:", add_block_response)

# Send the transaction
transaction_response = send_command("TRANSACTION Pranjal Aditya 100")
print("Response from adding block:", add_block_response)

# Print the entire blockchain
print_chain_response = send_command("PRINT_CHAIN")
print("Blockchain content:\n", print_chain_response)
