const Web3 = require('web3'),
  fs = require('fs');

const ABI_STAKING = require('./build/abi/Staking.json');
const ABI_GOVERNANCE = require('./build/abi/Governance.json');
const ABI_RUNTIME_UPGRADE = require('./build/abi/RuntimeUpgrade.json');

const askFor = async (question) => {
  return new Promise(resolve => {
    const readline = require('readline');
    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout
    });
    rl.question(question, (value) => {
      resolve(value);
      rl.close();
    });
  })
}

const readByteCodeForAddress = address => {
  const artifactPaths = {
    '0x0000000000000000000000000000000000001000': './build/contracts/Staking.json',
    '0x0000000000000000000000000000000000001001': './build/contracts/SlashingIndicator.json',
    '0x0000000000000000000000000000000000001002': './build/contracts/SystemReward.json',
    '0x0000000000000000000000000000000000007001': './build/contracts/StakingPool.json',
    '0x0000000000000000000000000000000000007002': './build/contracts/Governance.json',
    '0x0000000000000000000000000000000000007003': './build/contracts/ChainConfig.json',
    '0x0000000000000000000000000000000000007004': './build/contracts/RuntimeUpgrade.json',
    '0x0000000000000000000000000000000000007005': './build/contracts/DeployerProxy.json',
  }
  const filePath = artifactPaths[address]
  if (!filePath) throw new Error(`There is no artifact for the address: ${address}`)
  const {deployedBytecode} = JSON.parse(fs.readFileSync(filePath, 'utf8'))
  return deployedBytecode
}

const sleepFor = async ms => {
  return new Promise(resolve => setTimeout(resolve, ms))
}

const STAKING_ADDRESS = '0x0000000000000000000000000000000000001000';
const GOVERNANCE_ADDRESS = '0x0000000000000000000000000000000000007002';
const RUNTIME_UPGRADE_ADDRESS = '0x0000000000000000000000000000000000007004';

const proposalStates = ['Pending', 'Active', 'Canceled', 'Defeated', 'Succeeded', 'Queued', 'Expired', 'Executed'];

(async () => {
  const web3 = new Web3('https://rpc.dev-02.bas.ankr.com/');
  const staking = new web3.eth.Contract(ABI_STAKING, STAKING_ADDRESS);
  const governance = new web3.eth.Contract(ABI_GOVERNANCE, GOVERNANCE_ADDRESS);
  const runtimeUpgrade = new web3.eth.Contract(ABI_RUNTIME_UPGRADE, RUNTIME_UPGRADE_ADDRESS);
  // make sure we have enough private keys
  const keystoreKeys = {}
  const keystorePassword = fs.readFileSync('./password.txt', 'utf8')
  console.log(`Decrypting keystore`);
  for (const filePath of fs.readdirSync('./keystore', 'utf8')) {
    const [address] = filePath.match(/([\da-f]{40})/ig);
    console.log(` ~ decrypting account 0x${address}`);
    keystoreKeys[`0x${address}`.toLowerCase()] = web3.eth.accounts.decrypt(JSON.parse(fs.readFileSync(`./keystore/${filePath}`, 'utf8')), keystorePassword);
  }
  const activeValidatorSet = await staking.methods.getValidators().call();
  for (const validatorAddress of activeValidatorSet) {
    if (!keystoreKeys[validatorAddress.toLowerCase()]) {
      throw new Error(`Unable to find private key in keystore for address: ${validatorAddress}`)
    }
  }
  const someValidator = keystoreKeys[activeValidatorSet[0].toLowerCase()]
  if (!someValidator) {
    throw new Error(`There is no validators in the network, its not possible`)
  }
  // create new runtime upgrade proposal
  const contractAddress = await askFor('What address you\'d like to upgrade? '),
    byteCode = readByteCodeForAddress(contractAddress)
  const desc = `Runtime upgrade for the smart contract (${new Date().getTime()})`;
  const upgradeCall = runtimeUpgrade.methods.upgradeSystemSmartContract(contractAddress, byteCode, '0x').encodeABI(),
    governanceCall = governance.methods.propose([RUNTIME_UPGRADE_ADDRESS], ['0x00'], [upgradeCall], desc).encodeABI()
  const signTx = async (account, {to, data}) => {
    const nonce = await web3.eth.getTransactionCount(account.address),
      chainId = await web3.eth.getChainId()
    const txOpts = {
      from: account.address,
      gas: 2_000_000,
      gasPrice: 5e9,
      nonce: nonce,
      to: to,
      data: data,
      chainId: chainId,
    }
    await web3.eth.call(txOpts)
    return account.signTransaction(txOpts)
  }
  const {rawTransaction, transactionHash} = await signTx(someValidator, {
    to: GOVERNANCE_ADDRESS,
    data: governanceCall,
  });
  console.log(`Creating proposal: ${transactionHash}`);
  const proposeReceipt = await web3.eth.sendSignedTransaction(rawTransaction);
  const proposalId = proposeReceipt.logs[0].data.substring(0, 66)
  // let's vote for this proposal using all our validators
  console.log(`Waiting for the proposal become active`);
  while (true) {
    const state = await governance.methods.state(proposalId).call(),
      status = proposalStates[Number(state)];
    if (status === 'Active') {
      console.log(`Proposal is active, we can start voting process`);
      break;
    } else if (status !== 'Pending') {
      console.error(`Incorrect proposal status: ${status}`)
      return;
    }
    await sleepFor(1_000)
  }
  console.log(`Voting for the proposal (${proposalId}):`);
  for (const validatorAddress of activeValidatorSet) {
    const account = keystoreKeys[validatorAddress.toLowerCase()],
      castCall = governance.methods.castVote(proposalId, '1').encodeABI()
    const {rawTransaction, transactionHash} = await signTx(account, {
      to: GOVERNANCE_ADDRESS,
      data: castCall,
    })
    console.log(` ~ validator ${validatorAddress} is voting: ${transactionHash}`)
    await web3.eth.sendSignedTransaction(rawTransaction)
  }
  // now we can execute the proposal
  while (true) {
    const currentBlock = await web3.eth.getBlockNumber()
    const state = await governance.methods.state(proposalId).call(),
      status = proposalStates[Number(state)];
    const deadline = await governance.methods.proposalDeadline(proposalId).call();
    console.log(`Current proposal status is: ${status}, current block is: ${currentBlock} deadline is: ${deadline}, elapsed: ${deadline - currentBlock}`)
    switch (status) {
      case 'Pending':
      case 'Active': {
        break;
      }
      case 'Succeeded': {
        const executeCall = governance.methods.execute([RUNTIME_UPGRADE_ADDRESS], ['0x00'], [upgradeCall], web3.utils.keccak256(desc)).encodeABI()
        const {rawTransaction, transactionHash} = await signTx(someValidator, {
          to: GOVERNANCE_ADDRESS,
          data: executeCall,
        });
        console.log(`Executing proposal: ${transactionHash}`);
        await web3.eth.sendSignedTransaction(rawTransaction);
        break;
      }
      case 'Executed': {
        console.log(`Proposal was successfully executed`);
        return;
      }
      default: {
        console.error(`Incorrect proposal status, upgrade failed: ${status}, exiting`)
        return;
      }
    }
    await sleepFor(3_000)
  }
})();