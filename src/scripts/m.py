import socket

def send_command(command):
    host = '192.168.1.10'   # Ensure this matches the IP address of the device your rust node is live on
    port = 8080  # Ensure this matches the port your node is listening on

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((host, port))
        s.sendall((command + '\n').encode())  # Ensure command ends with newline
        data = s.recv(8192)  # Adjust buffer size if needed

    return data.decode()

# # Add a block to the blockchain
# add_block_response = send_command("ADD_BLOCK Your block D")
# print("Response from adding block:", add_block_response)



# Add Users
print("Adding Users...")
add_user_response = send_command("ADD_USER Nikhil 1000")
print(add_user_response)

add_user_response = send_command("ADD_USER Pranjal 1000")
print(add_user_response)

add_user_response = send_command("ADD_USER Aditya 1000")
print(add_user_response)

add_user_response = send_command("ADD_USER Shashank 10")
print(add_user_response)

add_user_response = send_command("ADD_USER Pawan 100000")
print(add_user_response)

# Send a transaction from Pawan to Nikhil
print("Pawan initiates a transaction to Nikhil...")
transaction_response = send_command("TRANSACTION TOKEN Pawan Nikhil 100")
print("Response from adding transaction:", transaction_response)

# View pending transactions for Nikhil
print("Nikhil views pending transactions...")
pending_transactions = send_command("VIEW_PENDING_TRANSACTIONS Nikhil")
print(pending_transactions)

# Extract Transaction ID from the response
lines = pending_transactions.strip().split('\n')
transaction_id = ''
for line in lines:
    if "Pending Transaction ID" in line:
        parts = line.split(',')
        for part in parts:
            if "Pending Transaction ID" in part:
                transaction_id = part.split(':')[1].strip()
                break

if not transaction_id:
    print("Failed to retrieve transaction ID.")
else:
    print(f"Transaction ID: {transaction_id}")
    # Nikhil confirms the transaction
    print("Nikhil confirms the transaction...")
    confirm_response = send_command(f"CONFIRM_TRANSACTION Nikhil {transaction_id}")
    print(confirm_response)

    # Check balances after confirmation
    print("Checking Balances after transaction...")
    balance_response = send_command("CHECK_BALANCE Pawan")
    print("Balance check:", balance_response)
    balance_response = send_command("CHECK_BALANCE Nikhil")
    print("Balance check:", balance_response)

    # Fetch User DAGs
    print("Fetching User DAGs for Pawan and Nikhil...")
    user_dags_response = send_command("FETCH_USER_DAGS Pawan Nikhil")
    print("Fetched User DAGs:\n", user_dags_response)

    # Query the transaction
    query_response = send_command(f"QUERY_TRANSACTION {transaction_id}")
    print("Query Transaction Response:", query_response)

    # Verify the transaction
    verify_response = send_command(f"VERIFY_TRANSACTION {transaction_id}")
    print("Verify Transaction Response:", verify_response)
