import { votingSuite } from "./vote-market-voting.suite";
import { rewardsSuite } from "./vote-market-rewards.suite";
import { RunCfg } from "./test-config";
import dotenv from "dotenv";
dotenv.config();

const run1: RunCfg = {
    rewardStyle: "airdrop",
    key1: process.env.KEY_PATH!,
    key2: process.env.KEY_PATH2!,
    config: process.env.CONFIG1!,
    rewardConfig: process.env.REWARD_CONFIG1};
const run2: RunCfg = {
    rewardStyle: "rewardAccumulator",
    key1: process.env.KEY_PATH3!,
    key2: process.env.KEY_PATH4!,
    config: process.env.CONFIG2!,
    rewardConfig: process.env.REWARD_CONFIG2};

describe("vote-market full runs", () => {
    // Order is exactly controlled here:
    votingSuite(run1);
    rewardsSuite(run1);

    votingSuite(run2);
    rewardsSuite(run2);
});
