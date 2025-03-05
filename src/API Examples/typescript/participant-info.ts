import {
  Participant,
  ParticipantDemographics,
  ParticipantHealthOverview,
} from "./types/participant.ts";

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
    message: "What infomation do you want?",
    type: Checkbox,
    options: ["general", "demographics", "health-overview"],
    default: ["general"],
  },
  {
    name: "participantId",
    message: `Enter the ID of the participant you want to fetch`,
    type: Number,
    default: 1,
  },
]);
const participantId = promptResponse.participantId ?? 1;
const stats = promptResponse.stat ? promptResponse.stat : ["general"];
// Make a requests
// Scalar: https://cs-25-303.wyatt-herkamp.dev/scalar#tag/participant-statistics

for (const statistic of stats) {
  if (statistic === "general") {
    const general = (await client.getJson(
      `/api/participant/get/${participantId}`
    )) as Participant;
    console.log("General: ", general);
  } else if (statistic === "demographics") {
    const demographics = (await client.getJson(
      `/api/participant/get/${participantId}/demographics`
    )) as ParticipantDemographics;

    console.log("Demographics: ", demographics);
  } else if (statistic === "health-overview") {
    const healthOverview = (await client.getJson(
      `/api/participant/get/${participantId}/health_overview`
    )) as ParticipantHealthOverview;

    console.log("Health Overview: ", healthOverview);
  }
}
