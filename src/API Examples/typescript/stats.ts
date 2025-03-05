import {
  BloodPressureStats,
  GlucoseEntry,
  WeightEntry,
} from "./types/stats.ts";
import { PaginatedResponse } from "./types/RequestTypes.ts";
import { CS25Client } from "./apiCore.ts";
import { parseFlags } from "@cliffy/flags";
import { User } from "./types/user.ts";
import { PRIMARY_CLI_FLAGS } from "./cli.ts";
import { prompt, Number, Checkbox } from "@cliffy/prompt";

const flags = parseFlags(Deno.args, {
  flags: [...PRIMARY_CLI_FLAGS],
});

const API_URL = flags.flags["api"] as string;

const client = new CS25Client(API_URL);
const user: User = await client.login(
  flags.flags["username"] as string,
  flags.flags["password"] as string
);
console.log(`You are logged in as ${user.username} on ${API_URL}`);

const promptResponse = await prompt([
  {
    name: "stat",
    message: "What statistic data points do you want?",
    type: Checkbox,
    options: ["blood-pressure", "weight", "glucose"],
    default: ["blood-pressure"],
  },
  {
    name: "participantId",
    message: `Enter the ID of the participant you want to fetch`,
    type: Number,
    default: 1,
  },
]);
const participantId = promptResponse.participantId ?? 1;
const stats = promptResponse.stat ? promptResponse.stat : ["blood-pressure"];
// Make a requests
// Scalar: https://cs-25-303.wyatt-herkamp.dev/scalar#tag/participant-statistics

for (const statistic of stats) {
  if (statistic === "blood-pressure") {
    const bpHistoryRequest = (await client.getJson(
      `/api/participant/stats/bp/history/${participantId}`
    )) as PaginatedResponse<BloodPressureStats>;

    console.log("Number of Elements in Total: ", bpHistoryRequest.total);
    console.log("Number of Pages: ", bpHistoryRequest.total_pages);
    for (const stats of bpHistoryRequest.data) {
      console.log("Stat: ", stats);
    }
  } else if (statistic === "weight") {
    const weightHistory = (await client.getJson(
      `/api/participant/stats/weight/history/${participantId}`
    )) as PaginatedResponse<WeightEntry>;

    console.log("Number of Elements in Total: ", weightHistory.total);
    console.log("Number of Pages: ", weightHistory.total_pages);
    console.log("Stat: ", weightHistory.data);
  } else if (statistic === "glucose") {
    const bloodGlucose = (await client.getJson(
      `/api/participant/stats/glucose/history/${participantId}`
    )) as PaginatedResponse<GlucoseEntry>;

    console.log("Number of Elements in Total: ", bloodGlucose.total);
    console.log("Number of Pages: ", bloodGlucose.total_pages);
    for (const stats of bloodGlucose.data) {
      console.log("Stat: ", stats);
    }
  }
}
