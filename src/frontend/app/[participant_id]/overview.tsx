import { useLocalSearchParams, useRouter } from 'expo-router';
import React, { useState, useEffect } from 'react';
import { View, Text } from 'react-native';
import LabelAndItem from '~/components/LabelAndItem';

import ProtectedRoute from '~/components/ProtectedRoute';

import api from '~/lib/api';

import { Participant, ParticipantStatus } from '~/lib/types/participant';
import { BoxHeader, participantOverViewStyles as styles } from '~/components/participant/overview';
import { HealthOverviewBox } from '~/components/participant/overview/health';
import { ParticipantDemographicsBox } from '~/components/participant/overview/demographics';
import { NoDataScreen } from '~/components/NoDataScreen';

export default function PatientInfo() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();
  const participantNumberId = Number.parseInt(participant_id);
  const router = useRouter();
  const [participant, setParticipant] = useState<Participant | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchPatient = async () => {
    try {
      const patient = await api.participants.fetchById(participantNumberId);
      setParticipant(patient);
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
        <ParticipantView participant={participant} />
      </View>
    </ProtectedRoute>
  );
}
function ParticipantView({ participant }: { participant: Participant | undefined }) {
  if (!participant) {
    return (
      <NoDataScreen title="No Participant Found" subtitle="No participant found with that ID" />
    );
  }
  return (
    <View>
      <View style={styles.fullWidth}>
        <Text style={styles.participantName}>
          {participant.first_name} {participant.last_name}
        </Text>
      </View>
      <View style={styles.flexRowWrap}>
        <ParticipantBox {...participant} />
        <HealthOverviewBox particpantId={participant.id} />
        <ParticipantDemographicsBox particpantId={participant.id} />
      </View>
    </View>
  );
}

function ParticipantBox(participant: Participant) {
  return (
    <View style={styles.box}>
      <BoxHeader title="Participant" />
      <View style={styles.marginBottom}>
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
