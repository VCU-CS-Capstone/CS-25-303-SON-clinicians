import { ResearcherParticipant } from "./types/researcher.ts";
import { PaginatedResponse } from "./types/RequestTypes.ts";
import { CS25Client } from "./apiCore.ts";
import { parseFlags } from "@cliffy/flags";
import { User } from "./types/user.ts";
import { PRIMARY_CLI_FLAGS } from "./cli.ts";

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
const foundParticipants = (await client.postJsonJsonBody(
  "/api/researcher/query",
  {
    age: ">25",
    gender: "male",
  }
)) as PaginatedResponse<ResearcherParticipant>;

console.log("Total Found Participants", foundParticipants.total);
console.log("Number Of Pages", foundParticipants.total_pages);

for (const participant of foundParticipants.data) {
  console.log(participant);
}
