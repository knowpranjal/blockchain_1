import socket

def send_command(command):
    host = '192.168.1.14'   # Ensure this matches the IP address of the device your rust node is live on
    port = 8080  # Ensure this matches the port your node is listening on

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((host, port))
        s.sendall((command + '\n').encode())  # Ensure command ends with newline
        data = s.recv(8192)  # Adjust buffer size if needed

    return data.decode()


# # Add User Alice with an initial balance of 1000
# alice_response = send_command("ADD_USER Alice 1000")
# print(alice_response)

# # Add User Bob with an initial balance of 500
# bob_response = send_command("ADD_USER Bob 500")
# print(bob_response)


# # Check balance for Alice
# alice_bal_response = send_command("CHECK_BALANCE Alice")
# print(alice_bal_response)

# # Check balance for Bob
# bob_bal_response = send_command("CHECK_BALANCE Bob")
# print(bob_bal_response)



tran_response = send_command("TRANSACTION TOKEN Bob Alice 150")
print(tran_response)


# # Check balance for Alice
# alice_bal_response = send_command("CHECK_BALANCE Alice")
# print(alice_bal_response)

# # Check balance for Bob
# bob_bal_response = send_command("CHECK_BALANCE Bob")
# print(bob_bal_response)



# # Check balance for Alice
# alice_bal_response = send_command("CHECK_BALANCE Alice")
# print(alice_bal_response)

# # Check balance for Bob
# bob_bal_response = send_command("CHECK_BALANCE Bob")
# print(bob_bal_response)


# response = send_command("PRINT_DAG")
# print(response)



