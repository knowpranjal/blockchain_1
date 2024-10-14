import socket
import time

def send_command(command):
    HOST = 'localhost'  # The server's hostname or IP address
    PORT = 8080         # The port used by the server

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((HOST, PORT))
        s.sendall((command + '\n').encode())
        s.shutdown(socket.SHUT_WR)  # Indicate that we're done sending
        response = b''
        while True:
            data = s.recv(4096)
            if not data:
                break
            response += data
    return response.decode()

# Add Users
print("Adding Users...")
response = send_command("ADD_USER Alice 1000")
print(response)

response = send_command("ADD_USER Bob 500")
print(response)

# Check Balances
print("Checking Balances...")
response = send_command("CHECK_BALANCE Alice")
print(response)

response = send_command("CHECK_BALANCE Bob")
print(response)

# Alice initiates a transaction to Bob
print("Alice initiates a transaction to Bob...")
response = send_command("TRANSACTION TOKEN Alice Bob 200")
print(response)

# Bob views pending transactions
print("Bob views pending transactions...")
response = send_command("VIEW_PENDING_TRANSACTIONS Bob")
print(response)

# Extract Transaction ID from the pending transaction
lines = response.strip().split('\n')
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

    # Bob confirms the transaction
    print("Bob confirms the transaction...")
    response = send_command(f"CONFIRM_TRANSACTION Bob {transaction_id}")
    print(response)

    # Check Balances again
    print("Checking Balances after transaction confirmation...")
    response = send_command("CHECK_BALANCE Alice")
    print(response)

    response = send_command("CHECK_BALANCE Bob")
    print(response)

    # Verify the transaction
    print("Verifying the transaction...")
    response = send_command(f"VERIFY_TRANSACTION {transaction_id}")
    print(response)

    # Print Global DAG
    print("Printing Global DAG...")
    response = send_command("PRINT_DAG")
    print(response)

    # Alice prints her local DAG
    print("Alice prints her local DAG...")
    response = send_command("PRINT_USER_DAG Alice")
    print(response)

    # Bob prints his local DAG
    print("Bob prints his local DAG...")
    response = send_command("PRINT_USER_DAG Bob")
    print(response)

# Bob initiates a transaction to Alice
print("Bob initiates a transaction to Alice...")
response = send_command("TRANSACTION TOKEN Bob Alice 100")
print(response)

# Alice views pending transactions
print("Alice views pending transactions...")
response = send_command("VIEW_PENDING_TRANSACTIONS Alice")
print(response)

# Extract Transaction ID from the pending transaction
lines = response.strip().split('\n')
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

    # Alice rejects the transaction
    print("Alice rejects the transaction...")
    response = send_command(f"REJECT_TRANSACTION Alice {transaction_id}")
    print(response)

    # Check Balances after rejection
    print("Checking Balances after transaction rejection...")
    response = send_command("CHECK_BALANCE Alice")
    print(response)

    response = send_command("CHECK_BALANCE Bob")
    print(response)

# View pending transactions to ensure it's removed
print("Bob views pending transactions after rejection...")
response = send_command("VIEW_PENDING_TRANSACTIONS Bob")
print(response)

# Validate Alice's local DAG
print("Validating Alice's local DAG...")
response = send_command("VALIDATE_LOCAL_DAG Alice")
print(response)

# Validate Bob's local DAG
print("Validating Bob's local DAG...")
response = send_command("VALIDATE_LOCAL_DAG Bob")
print(response)

# Fetch User DAGs
print("Fetching User DAGs for Alice and Bob...")
response = send_command("FETCH_USER_DAGS Alice Bob")
print(response)
