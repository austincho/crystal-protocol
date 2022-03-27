import { LCDClient, MsgExecuteContract, Fee } from "@terra-money/terra.js";
import { contractAddress } from "./address";

// ==== utils ====

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const until = Date.now() + 1000 * 60 * 60;
const untilInterval = Date.now() + 1000 * 60;

const _exec =
  (msg, fee = new Fee(200000, { uluna: 10000 })) =>
  async (wallet) => {
    const lcd = new LCDClient({
      URL: wallet.network.lcd,
      chainID: wallet.network.chainID,
    });

    const { result } = await wallet.post({
      fee,
      msgs: [
        new MsgExecuteContract(
          wallet.walletAddress,
          contractAddress(wallet),
          msg,
        ),
      ],
    });

    while (true) {
      try {
        return await lcd.tx.txInfo(result.txhash);
      } catch (e) {
        if (Date.now() < untilInterval) {
          await sleep(500);
        } else if (Date.now() < until) {
          await sleep(1000 * 10);
        } else {
          throw new Error(
            `Transaction queued. To verify the status, please check the transaction hash: ${result.txhash}`
          );
        }
      }
    }
  };

// ==== execute contract ====

export const transferOption = async (wallet, recipient) => _exec({ transfer_option: { recipient : {recipient} }});
export const fundPremium = _exec({ fund_premium: {} });
export const underwriteOption = async(wallet, recipient) => _exec({ underwrite_option: {} });
export const executeOption = _exec({ increment: {} });
export const withdrawExpiredOption = _exec({ increment: {} });
export const withdrawUnlockedOption = _exec({ increment: {} });

export const fund = async (wallet, fee, msg, coins) => {
    const lcd = new LCDClient({
        URL: wallet.network.lcd,
        chainID: wallet.network.chainID,
    });

    const {result} = await wallet.post({
        fee,
        msgs: [
            new MsgExecuteContract(
                wallet.walletAddress,
                contractAddress(wallet),
                msg,
                coins
            ),
        ],
    });
    while (true) {
        try {
            return await lcd.tx.txInfo(result.txhash);
        } catch (e) {
            if (Date.now() < untilInterval) {
                await sleep(500);
            } else if (Date.now() < until) {
                await sleep(1000 * 10);
            } else {
                throw new Error(
                    `Transaction queued. To verify the status, please check the transaction hash: ${result.txhash}`
                );
            }
        }
    }
};
