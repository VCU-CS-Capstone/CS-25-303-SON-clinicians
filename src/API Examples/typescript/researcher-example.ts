import { ResearcherParticipant } from "./types/researcher.ts";
import { PaginatedResponse } from "./types/RequestTypes.ts";
import { CS25Client } from "./apiCore.ts";
import { parseFlags } from "@cliffy/flags";
import { User } from "./types/user.ts";
import { PRIMARY_CLI_FLAGS } from "./cli.ts";
import { prompt, Checkbox } from "@cliffy/prompt";

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
    name: "queryExample",
    message: "What Query Example do you want to test?",
    type: Checkbox,
    options: ["bmi", "blood-pressure", "glucose", "generic"],
    default: ["generic", "blood-pressure"],
  },
]);
const queryExamples = promptResponse.queryExample
  ? promptResponse.queryExample
  : ["generic", "blood-pressure"];

for (const queryExample of queryExamples) {
  if (queryExample === "generic") {
    const foundParticipants = (await client.postJsonJsonBody(
      "/api/researcher/query",
      {
        age: ">25",
        gender: "male",
      }
    )) as PaginatedResponse<ResearcherParticipant>;
    displayResponse(foundParticipants);
  } else if (queryExample === "bmi") {
    const foundParticipants = (await client.postJsonJsonBody(
      "/api/researcher/query",
      {
        bmi: ">25",
      }
    )) as PaginatedResponse<ResearcherParticipant>;
    displayResponse(foundParticipants);
  } else if (queryExample === "blood-pressure") {
    const foundParticipants = (await client.postJsonJsonBody(
      "/api/researcher/query",
      {
        blood_pressure: {
          systolic: ">120",
          diastolic: ">80",
        },
      }
    )) as PaginatedResponse<ResearcherParticipant>;
    displayResponse(foundParticipants);
  } else if (queryExample === "glucose") {
    const foundParticipants = (await client.postJsonJsonBody(
      "/api/researcher/query",
      {
        glucose: {
          glucose: ">120",
          fasted_atleast_2_hours: true,
        },
      }
    )) as PaginatedResponse<ResearcherParticipant>;
    displayResponse(foundParticipants);
  }
}

function displayResponse(response: PaginatedResponse<ResearcherParticipant>) {
  console.log("Total Found Participants", response.total);
  console.log("Number Of Pages", response.total_pages);

  for (const participant of response.data) {
    console.log(participant);
  }
}
