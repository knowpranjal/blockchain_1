import socket

def send_command(command):
    host = '192.168.0.121'   # Ensure this matches the IP address of the device your rust node is live on
    port = 8080  # Ensure this matches the port your node is listening on

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((host, port))
        s.sendall((command + '\n').encode())  # Ensure command ends with newline
        data = s.recv(8192)  # Adjust buffer size if needed

    return data.decode()

# # Add a block to the blockchain
# add_block_response = send_command("ADD_BLOCK Your block D")
# print("Response from adding block:", add_block_response)



add_user_command = send_command("ADD_USER Nikhil 1000")
print(add_user_command)

add_user_command = send_command("ADD_USER Pranjal 1000")
print(add_user_command)

add_user_command = send_command("ADD_USER Aditya 1000")
print(add_user_command)

add_user_command = send_command("ADD_USER Shashank 10")
print(add_user_command)

add_user_command = send_command("ADD_USER Pawan 100000")
print(add_user_command)

# Send multiple transactions
transaction_response = send_command("TRANSACTION " 
                                    "TOKEN Pawan Nikhil 100")
print("Response from adding transactions:", transaction_response)

transaction_id = "9c11a164-4261-4990-97e6-5de7f168c5b4"  # Replace with an actual transaction ID

# Query the transaction
query_response = send_command(f"QUERY_TRANSACTION {transaction_id}")
print("Query Transaction Response:", query_response)

verify_response = send_command(f"VERIFY_TRANSACTION {transaction_id}")
print("Verify Transaction Response:", verify_response)

# Print the USer DAG
user_DAG_response = send_command("PRINT_USER_DAG Pranjal")
print("Response from adding block:", user_DAG_response)

# Print the User2 DAG
user_DAG_response = send_command("PRINT_USER_DAG Nikhil")
print("Response from adding block:", user_DAG_response)

# Print the User2 DAG
user_DAG_response = send_command("PRINT_USER_DAG Pawan")
print("Response from adding block:", user_DAG_response)

# Print the entire blockchain
print_chain_response = send_command("PRINT_DAG")
print("Blockchain content:\n", print_chain_response)


# Check balances before transactions
balance_response = send_command("CHECK_BALANCE Nikhil")
print("Balance check:", balance_response)

balance_response = send_command("CHECK_BALANCE Pranjal")
print("Balance check:", balance_response)

balance_response = send_command("CHECK_BALANCE Aditya")
print("Balance check:", balance_response)

balance_response = send_command("CHECK_BALANCE Nikhil")
print("Balance check:", balance_response)

balance_response = send_command("CHECK_BALANCE Pawan")
print("Balance check:", balance_response)


user_names = ["Pawan", "Nikhil"]
fetch_dags_command = "FETCH_USER_DAGS " + " ".join(user_names)
user_dags_response = send_command(fetch_dags_command)
print("Fetched User DAGs:\n", user_dags_response)