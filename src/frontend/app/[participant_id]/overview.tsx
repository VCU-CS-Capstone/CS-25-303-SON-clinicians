import { useLocalSearchParams, useRouter } from 'expo-router';
import React, { useState, useEffect } from 'react';
import { View, Text } from 'react-native';
import { Button } from '~/components/Button';
import LabelAndItem from '~/components/LabelAndItem';

import ProtectedRoute from '~/components/ProtectedRoute';
import { HamburgerMenu } from '~/components/menus/hamurger';
import { HamburgerOption } from '~/components/menus/hamurger/HamburgerOption';
import { HealthInsuranceSelector, ShowInsurances } from '~/components/participant/HealthInsurance';
import api from '~/lib/api';
import {
  Participant,
  ParticipantDemographics,
  ParticipantHealthOverview,
  ParticipantStatus,
} from '~/lib/types/participant';

export default function PatientInfo() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();
  const participantNumberId = Number.parseInt(participant_id);
  const router = useRouter();
  const [participant, setParticipant] = useState<Participant | undefined>(undefined);
  const [participantHealthOverview, setParticipantHealthOverview] = useState<
    ParticipantHealthOverview | undefined
  >(undefined);
  const [participantDemographics, setParticipantDemographics] = useState<
    ParticipantDemographics | undefined
  >(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchPatient = async () => {
    try {
      const patient = await api.participants.fetchById(participantNumberId);
      const healthOverview = await api.participants.fetchHealthOverview(participantNumberId);
      const demographics = await api.participants.fetchDemographic(participantNumberId);
      setParticipant(patient);
      setParticipantHealthOverview(healthOverview);
      setParticipantDemographics(demographics);
      setError(undefined);
      setLoading(false);
    } catch (e: any) {
      setError(e.message as string);
      setLoading(false);
    }
  };
  useEffect(() => {
    fetchPatient();
  }, []);

  return (
    <ProtectedRoute>
      <View>
        {error ? <Text>{error}</Text> : null}
        <ParticipantView
          participant={participant}
          healthOverview={participantHealthOverview}
          demographics={participantDemographics}
        />
      </View>
    </ProtectedRoute>
  );
}
function ParticipantView({
  participant,
  healthOverview,
  demographics,
}: {
  participant: Participant | undefined;
  healthOverview: ParticipantHealthOverview | undefined;
  demographics: ParticipantDemographics | undefined;
}) {
  if (!participant) {
    // TODO: 404 page
    return null;
  }
  return (
    <View>
      <View className="w-full">
        <Text className="text-4xl font-bold">
          {participant.first_name} {participant.last_name}
        </Text>
      </View>
      <View className="flex flex-row flex-wrap">
        <ParticipantBox {...participant} />
        <HealthOverviewBox healthOverview={healthOverview} />
        <DemographicsBox demographics={demographics} />
      </View>
    </View>
  );
}
function BoxHeader({ title }: { title: string }) {
  return (
    <View className="flex flex-row justify-between border-b p-4">
      <Text className="text-2xl font-bold">{title}</Text>
      <HamburgerMenu iconWidth={36} iconHeight={36}>
        <HamburgerOption title="Open Page" />
        <HamburgerOption title="Edit Page" />
      </HamburgerMenu>
    </View>
  );
}
function ParticipantBox(participant: Participant) {
  return (
    <View className="basis-1/2 border px-4">
      <BoxHeader title="Participant" />
      <View className="mb-4">
        <LabelAndItem label="Contact Info">
          <Text>{participant.phone_number_one}</Text>
        </LabelAndItem>
        <LabelAndItem label="Status">
          <Text>{ParticipantStatus.title(participant.status)}</Text>
        </LabelAndItem>
      </View>
    </View>
  );
}

function HealthOverviewBox({
  healthOverview,
}: {
  healthOverview: ParticipantHealthOverview | undefined;
}) {
  if (!healthOverview) {
    return (
      <View>
        <Text>Loading...</Text>
      </View>
    );
  }
  return (
    <View className="basis-1/2 border px-4">
      <BoxHeader title="Health Overview" />
      <View>
        <YesOrNo
          label="Takes More Than 5 Medications"
          value={healthOverview.takes_more_than_5_medications}
        />
        <YesOrNo
          label="Has Blood Pressure Cuff"
          value={healthOverview.takes_more_than_5_medications}
        />
      </View>
      <LabelAndItem label="Allergies">
        <Text>{healthOverview.reported_health_conditions || 'None'}</Text>
      </LabelAndItem>
      <LabelAndItem label="Reported Health Conditions">
        <Text>{healthOverview.reported_health_conditions || 'None'}</Text>
      </LabelAndItem>
    </View>
  );
}
function YesOrNo({ label, value }: { label: string; value?: boolean }) {
  return (
    <View className="mb-4">
      <Text className="text-2xl font-bold">
        {label}: {value ? 'Yes' : 'No'}
      </Text>
    </View>
  );
}
function DemographicsBox({ demographics }: { demographics: ParticipantDemographics | undefined }) {
  if (!demographics) {
    return (
      <View>
        <Text>Loading...</Text>
      </View>
    );
  }
  return (
    <View className="w-1/2 border px-4">
      <BoxHeader title="Demographics" />

      <View className="flex flex-row justify-between">
        <LabelAndItem label="Age">
          <Text>{demographics.age}</Text>
        </LabelAndItem>
        <YesOrNo label="Is Veteran" value={demographics.is_veteran} />
      </View>
      <LabelAndItem label="Health Insurace">
        <ShowInsurances insurances={demographics.health_insurance || []} />
      </LabelAndItem>
    </View>
  );
}
