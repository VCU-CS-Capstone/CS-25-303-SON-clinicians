import { useLocalSearchParams } from 'expo-router';
import React, { useState, useEffect } from 'react';
import { FlatList, Text, View } from 'react-native';
import { LocationName } from '~/components/LocationName';

import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import { RecentVisit } from '~/lib/types/participant';
import { StyleSheet } from 'react-native';
import { NoDataScreen } from '~/components/NoDataScreen';
import LabelAndItem from '~/components/LabelAndItem';

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
      <VisitsList visits={visits} />
    </ProtectedRoute>
  );
}

function VisitsList({ visits }: { visits: RecentVisit[] | undefined }) {
  if (!visits || visits.length === 0) {
    return <NoDataScreen title="No Visits Found" subtitle="No visits found for Participant" />;
  }
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
    <View style={styles.visitContainer}>
      <Text style={styles.dateOfVisit}>
        {new Date(visit.date_of_visit).toLocaleDateString()} - {visit.visit_type}{' '}
      </Text>
      <LabelAndItem label="Location">
        <LocationName locationId={visit.location} />
      </LabelAndItem>
    </View>
  );
}

const styles = StyleSheet.create({
  visitContainer: {
    marginLeft: 16,
    marginBottom: 16,
    borderWidth: 2,
    width: '25%',
    padding: 16,
  },
  dateOfVisit: {
    fontSize: 20,
    fontWeight: 'bold',
  },
});
