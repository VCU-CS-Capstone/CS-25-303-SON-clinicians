import { useLocalSearchParams, useRouter } from 'expo-router';
import React, { useState, useEffect } from 'react';
import { View, Text } from 'react-native';
import { Button } from '~/components/Button';
import LabelAndItem from '~/components/LabelAndItem';

import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import { Participant } from '~/lib/types/participant';

export default function PatientInfo() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();
  const router = useRouter();
  const [participant, setParticipant] = useState<Participant | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchPatient = async () => {
    try {
      const patient = await api.participants.fetchById(Number.parseInt(participant_id));
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
      <Button title={'Refresh Page'} onPress={fetchPatient}>
        Refresh
      </Button>
      <View>
        {error ? <Text>{error}</Text> : null}
        {participant && participant.id !== undefined && <ShowParticipant {...participant} />}
      </View>
    </ProtectedRoute>
  );
}

function ShowParticipant(participant: Participant) {
  return (
    <View>
      <View className="flex  flex-row">
        <View className="mb-4">
          <LabelAndItem label="Name">
            <Text className="text-2xl font-bold">
              {participant.first_name} {participant.last_name}
            </Text>
          </LabelAndItem>
          <LabelAndItem label="Contact Info">
            <Text>{participant.phone_number_one}</Text>
          </LabelAndItem>
        </View>
      </View>
    </View>
  );
}
