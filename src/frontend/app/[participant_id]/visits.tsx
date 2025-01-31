import { useLocalSearchParams } from 'expo-router';
import React, { useState, useEffect } from 'react';
import { FlatList, Text, View } from 'react-native';
import { LocationName } from '~/components/LocationName';

import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import { RecentVisit } from '~/lib/types/participant';

export default function participant_visit_history() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();

  const [visits, setVisits] = useState<RecentVisit[] | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchVisits = async () => {
      try {
        const patient = await api.participants.getRecentVisits(Number.parseInt(participant_id));
        setVisits(patient);
        setError(undefined);
        setLoading(false);
      } catch (e: any) {
        setError(e.message as string);
        setLoading(false);
      }
    };
    fetchVisits();
  }, [participant_id]);

  return (
    <ProtectedRoute>
      <FlatList
        data={visits}
        renderItem={({ item }) => <VisitSummary visit={item} />}
        keyExtractor={(item) => item.id.toString()}
      />
    </ProtectedRoute>
  );
}
function VisitSummary({ visit }: { visit: RecentVisit }) {
  return (
    <View className="mb-4 border-2 border-solid border-red-100">
      <Text>{visit.date_of_visit}</Text>
      <Text>{visit.visit_type}</Text>
      <LocationName locationId={visit.location} />
    </View>
  );
}
