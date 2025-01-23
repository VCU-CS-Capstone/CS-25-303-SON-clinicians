import React, { useState, useEffect } from 'react';
import { View, Text } from 'react-native';
import { RefreshControl, ScrollView } from 'react-native-gesture-handler';
import LabelAndItem from '~/components/LabelAndItem';
import RecentVisits from '~/components/participant/RecentVisits';

import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import { Participant } from '~/lib/types/participant';

export default function PatientInfo() {
  const [participant, setParticipant] = useState<Participant | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchPatient = async () => {
    try {
      const patient = await api.participants.fetchById(1);
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
  const onRefresh = React.useCallback(() => {
    setLoading(true);
    fetchPatient();
  }, []);

  return (
    <ProtectedRoute>
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
        <RecentVisits participantId={participant.id} />
      </View>
    </View>
  );
}
