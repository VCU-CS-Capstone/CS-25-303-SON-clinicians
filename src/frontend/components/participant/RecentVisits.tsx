import React, { useEffect, useState } from 'react';
import { FlatList, View, Text } from 'react-native';
import api from '~/lib/api';
import { RecentVisit } from '~/lib/types/participant';

export default function RecentVisits({ participantId }: { participantId: number }) {
  const [visits, setVisits] = useState<RecentVisit[] | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchVisits = async () => {
    try {
      const patient = await api.participants.getRecentVisits(participantId);
      setVisits(patient);
      setError(undefined);
      setLoading(false);
    } catch (e: any) {
      setError(e.message as string);
      setLoading(false);
    }
  };
  useEffect(() => {
    fetchVisits();
  }, []);

  return (
    <FlatList
      data={visits}
      renderItem={({ item }) => <VisitSummary visit={item} />}
      keyExtractor={(item) => item.id.toString()}
    />
  );
}

function VisitSummary({ visit }: { visit: RecentVisit }) {
  return (
    <View className="mb-4 border-2 border-solid border-red-100">
      <Text>{visit.date_of_visit}</Text>
      <Text>{visit.visit_type}</Text>
      <Text>{visit.location}</Text>
    </View>
  );
}
