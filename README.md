<h3>Overview</h3>
This clent-server bundle is made for deploying simple distributed computation network based on ipv4.

The bundle consists of 3 binaries:
  1. Server
  2. Client
  3. Launcher (for updating the client)


<h3>Server</h3>
Server listens on 0.0.0.0:1337 for incoming client connections.
After a three-way-handshake (to check if connection is genuine) and an update client is added to pool of workers.

The three-way-handshake in normal mode goes as following:
1. Client TCP connects to port 1337
2. Server answers with b"master"
3. Client answers with b"normal"

Server also runs a web page via rocket as an interface for adding tasks and viewing their results.
A task consists of a shell script (will be executed) and an optional attachment (a single file).

When a new task is recieved, it is saved and added to a task queue.
A separate thread is running to assign tasks in queue to free workers from pool.
The task is sent to the worker and is executed.
After execution stdout and stdin are sent back to server and saved to add them to the task's web page.
The worker is freed ready for new tasks.


<h3>Client</h3>
Client's routine mainly consists of:
  1. Finding the server
  2. Connecting to the server
  3. Updating
  4. Recieving a task
  5. Executing the task
  6. Sending back results
  7. GOTO step 3

<h2>Finding the server</h2>
Client has a savedata file that contains server ip address.
If a connection to the address is unsuccessfull, client invalidates savedata address and bruteforces it.
Client tries connecting to random ip's in its subnet.
If through TWH it finds a server it succeeds.
Otherwise, if it has found another client, it asks via TWH for masters ip and tries it.
This feature was implemented specifically for networks with dynamic ip allocation.

<h2>Upadting</h2>
Client calculates hash of its binary and compares with the latest client hash from server.
If hashes don't match client requests a fresh binary. After writing the new binary and checking its hash
client terminates falling back to launcher, which finalizes update and restarts client.

<h2>Sending back results</h2>
After results (stoud and stderr of the script) are sent back to server client deletes task files (the script and attachment)
if retain_attachment flag is set the attachment is not deleted.

<h3>Launcher</h3>
Launches the client.
Depending on exit code may update it by swapping old client file with a new one (created by client),
then restart the client.

![Untitled-2023-11-11-1851](https://github.com/user-attachments/assets/84694be9-e8ff-437e-be96-7a3aaf04457b)
