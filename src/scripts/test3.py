import socket

def send_command(command):
    host = '192.168.1.12'   # Replace with your server's IP address
    port = 8080             # Ensure this matches the port your node is listening on

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((host, port))
        s.sendall((command + '\n').encode())  # Ensure command ends with newline
        response = ''
        while True:
            data = s.recv(4096)
            if not data:
                break
            response += data.decode()
    return response

# Add Users
print("Adding Users...")
alice_response = send_command("ADD_USER Alice 1000")
print(alice_response)

bob_response = send_command("ADD_USER Bob 500")
print(bob_response)

# Check Balances
print("Checking Balances...")
alice_balance = send_command("CHECK_BALANCE Alice")
print(alice_balance)

bob_balance = send_command("CHECK_BALANCE Bob")
print(bob_balance)

# Alice initiates a transaction to Bob
print("Alice initiates a transaction to Bob...")
transaction_response = send_command("TRANSACTION TOKEN Alice Bob 200")
print(transaction_response)

# Bob views pending transactions
print("Bob views pending transactions...")
pending_transactions = send_command("VIEW_PENDING_TRANSACTIONS Bob")
print(pending_transactions)

# Extract Transaction ID
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
    # Bob confirms the transaction
    print("Bob confirms the transaction...")
    confirm_response = send_command(f"CONFIRM_TRANSACTION Bob {transaction_id}")
    print(confirm_response)

    # Check Balances after transaction
    print("Checking Balances after transaction...")
    alice_balance = send_command("CHECK_BALANCE Alice")
    print(alice_balance)

    bob_balance = send_command("CHECK_BALANCE Bob")
    print(bob_balance)

    # Validate Alice's Local DAG
    print("Validating Alice's Local DAG...")
    validate_response = send_command("VALIDATE_LOCAL_DAG Alice")
    print(validate_response)

    # Validate Bob's Local DAG
    print("Validating Bob's Local DAG...")
    validate_response = send_command("VALIDATE_LOCAL_DAG Bob")
    print(validate_response)

# Print Global DAG
print("Printing Global DAG...")
dag_response = send_command("PRINT_DAG")
print(dag_response)

# Alice initiates another transaction to Bob
print("Alice initiates another transaction to Bob...")
transaction_response = send_command("TRANSACTION TOKEN Alice Bob 300")
print(transaction_response)

# Bob views pending transactions
print("Bob views pending transactions...")
pending_transactions = send_command("VIEW_PENDING_TRANSACTIONS Bob")
print(pending_transactions)

# Extract Transaction ID
lines = pending_transactions.strip().split('\n')
transaction_id_reject = ''
for line in lines:
    if "Pending Transaction ID" in line:
        parts = line.split(',')
        for part in parts:
            if "Pending Transaction ID" in part:
                transaction_id_reject = part.split(':')[1].strip()
                break

if not transaction_id_reject:
    print("Failed to retrieve transaction ID for rejection.")
else:
    print(f"Transaction ID for rejection: {transaction_id_reject}")
    # Bob rejects the transaction
    print("Bob rejects the transaction...")
    reject_response = send_command(f"REJECT_TRANSACTION Bob {transaction_id_reject}")
    print(reject_response)

    # Check Balances after rejection
    print("Checking Balances after rejection...")
    alice_balance = send_command("CHECK_BALANCE Alice")
    print(alice_balance)

    bob_balance = send_command("CHECK_BALANCE Bob")
    print(bob_balance)

# Fetch User DAGs
print("Fetching User DAGs for Alice and Bob...")
user_dags = send_command("FETCH_USER_DAGS Alice Bob")
print(user_dags)

# Verify a Transaction
print("Verifying the first confirmed transaction...")
if transaction_id:
    verify_response = send_command(f"VERIFY_TRANSACTION {transaction_id}")
    print(verify_response)
else:
    print("No transaction ID to verify.")

# Bob initiates a transaction to Alice
print("Bob initiates a transaction to Alice...")
transaction_response = send_command("TRANSACTION TOKEN Bob Alice 100")
print(transaction_response)

# Alice views pending transactions
print("Alice views pending transactions...")
pending_transactions = send_command("VIEW_PENDING_TRANSACTIONS Alice")
print(pending_transactions)

# Extract Transaction ID
lines = pending_transactions.strip().split('\n')
transaction_id_alice = ''
for line in lines:
    if "Pending Transaction ID" in line:
        parts = line.split(',')
        for part in parts:
            if "Pending Transaction ID" in part:
                transaction_id_alice = part.split(':')[1].strip()
                break

if not transaction_id_alice:
    print("Failed to retrieve transaction ID from Alice's pending transactions.")
else:
    print(f"Transaction ID: {transaction_id_alice}")
    # Alice confirms the transaction
    print("Alice confirms the transaction...")
    confirm_response = send_command(f"CONFIRM_TRANSACTION Alice {transaction_id_alice}")
    print(confirm_response)

    # Check Balances after transaction
    print("Checking Balances after Bob's transaction...")
    alice_balance = send_command("CHECK_BALANCE Alice")
    print(alice_balance)

    bob_balance = send_command("CHECK_BALANCE Bob")
    print(bob_balance)

# Validate DAGs again
print("Validating Alice's Local DAG after Bob's transaction...")
validate_response = send_command("VALIDATE_LOCAL_DAG Alice")
print(validate_response)

print("Validating Bob's Local DAG after Bob's transaction...")
validate_response = send_command("VALIDATE_LOCAL_DAG Bob")
print(validate_response)
