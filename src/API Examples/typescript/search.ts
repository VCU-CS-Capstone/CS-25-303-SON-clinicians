import { ParticipantLookupResponse } from "./types/participant.ts";
import { PaginatedResponse } from "./types/RequestTypes.ts";

import { CS25Client } from "./apiCore.ts";
import { parseFlags } from "@cliffy/flags";
import { User } from "./types/user.ts";
import { PRIMARY_CLI_FLAGS } from "./cli.ts";
import { prompt, Checkbox, Input } from "@cliffy/prompt";

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
    name: "firstName",
    message: `First Name`,
    type: Input,
  },
  {
    name: "lastName",
    message: `Last Name`,
    type: Input,
  },

  {
    name: "program",
    message: "Program",
    type: Checkbox,
    options: ["RHWP", "MHWP"],
    maxOptions: 1,
  },
]);
function filterString(value: string | undefined) {
  if (!value) {
    return undefined;
  }
  if (value === "") {
    return undefined;
  }
  return value as string;
}
const firstName = filterString(promptResponse.firstName);
const lastName = filterString(promptResponse.lastName);
const program =
  promptResponse.program && promptResponse.program.length == 1
    ? promptResponse.program[0]
    : undefined;
// Make a requests
// Scalar: https://cs-25-303.wyatt-herkamp.dev/scalar#tag/participant-statistics

const foundParticipants = (await client.postJsonJsonBody(
  "api/participant/lookup",
  {
    first_name: firstName,
    last_name: lastName,
    program: program,
  }
)) as PaginatedResponse<ParticipantLookupResponse>;

console.log("Total Found Participants", foundParticipants.total);
console.log("Total Pages", foundParticipants.total_pages);

if (foundParticipants.total == 0) {
  console.log("No participants found");
} else {
  for (const participant of foundParticipants.data) {
    console.log(participant);
  }
}
