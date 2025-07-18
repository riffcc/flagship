### **Quickstart: Deploying a Flagship Instance with a Local Lens Node**

This guide will walk you through setting up a local Lens Node and deploying a Flagship instance that connects to it.

### **Prerequisites**

Before you begin, ensure you have the following installed:

*   [Node.js](https://nodejs.org/)
*   [pnpm](https://pnpm.io/installation)
*   [git](https://git-scm.com/downloads)
*
---

### **Step 1: Set Up and Run the Lens Node**

First, we will install and run the Lens Node on your local machine. This node will act as the backend for your Flagship instance.

1.  **Install the Lens Node globally:**
    ```bash
    pnpm install -g @riffcc/lens-node
    ```

2.  **Run the initial setup process:**
    ```bash
    lens-node setup
    ```

3.  **Start the node:**
    ```bash
    lens-node run
    ```

Once the node is running, you will see output similar to this. **Take note of your `Site Address` and the `Listening on` address that contains `/ws`**, as you will need them in the next step.

```bash
Node Directory: ~/.lens-node
Peer ID: 12D3KooWCkhrz3Kob1qLTMVFdbAJs3VnkgtR4XSXyUZsUDD4NC4H
Node Public Key: ed25119p/2ba30160b78da2b3ceecc6e4488735288fc30433de80f03eb1b4f9e1277aa5c2
Site Address: zb2rhf6jXPAeDtE77qrnqvb3tZFuiZ13dosxtshpDqjUa2coN
Listening on: [
"/ip4/127.0.0.1/tcp/8002/ws/p2p/12D3KooWCkhrz3Kob1qLTMVFdbAJs3VnkgtR4XSXyUZsUDD4NC4H",
"/ip4/127.0.0.1/tcp/8001/p2p/12D3KooWCkhrz3Kob1qLTMVFdbAJs3VnkgtR4XSXyUZsUDD4NC4H"
]
```

You now have a Lens Node running successfully. Keep this terminal window open.

---

### **Step 2: Deploy the Flagship Instance**

Next, we will clone the Flagship repository and configure it to connect to your local Lens Node.

1.  **Clone the repository and navigate into the directory:**
    ```bash
    git clone https://github.com/riffcc/flagship
    cd flagship
    ```

2.  **Install dependencies and create a local environment file by copying the example:**
    ```bash
    pnpm install
    cp .env.example .env
    ```

3.  **Configure the environment variables.** Open the newly created `.env` file and set the following variables using the values from your running Lens Node (from Step 1):

    ```dotenv
    # Paste the "Site Address" from your terminal output here
    VITE_SITE_ADDRESS=zb2rhf6jXPAeDtE77qrnqvb3tZFuiZ13dosxtshpDqjUa2coN

    # Paste the "Listening on" address that includes "/ws" here
    VITE_BOOTSTRAPPERS=/ip4/127.0.0.1/tcp/8002/ws/p2p/12D3KooWCkhrz3Kob1qLTMVFdbAJs3VnkgtR4XSXyUZsUDD4NC4H
    ```

    > **Note:** Only the listening address containing `/ws` (WebSocket) is compatible with browser connections.

4.  **Build and serve the Flagship application:**
    ```bash
    pnpm preview:web
    ```

5.  **Access the application** in your browser at: **http://localhost:4173/**

---

### **Step 3: Grant Admin Privileges**

To access the site management panel, you need to promote your account to an Admin.

1.  In your browser, navigate to your running Flagship instance. Click the **avatar icon** in the top-right corner, then select the **Account** tab.

2.  Find your **Public Key** and click on the field to copy it to your clipboard.

3.  Return to the terminal window where your Lens Node is running. It should display an interactive menu. Use the arrow keys to select the **`Authorize Account`** option and press Enter.

4.  Paste the Public Key you copied from the browser and press Enter.

5.  In the next step, select the **`Admin`** role and press Enter.

After a few moments, a new **Admin** tab will appear in the Flagship application's navigation bar. You have now successfully deployed a local Flagship instance and granted yourself administrative rights
