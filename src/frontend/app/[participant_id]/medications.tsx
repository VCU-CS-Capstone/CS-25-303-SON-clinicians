import { Ionicons } from '@expo/vector-icons';
import { useLocalSearchParams } from 'expo-router';
import { useEffect, useState } from 'react';
import { ActivityIndicator, FlatList, StatusBar, StyleSheet, Text, View } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
import { NoDataScreen } from '~/components/NoDataScreen';
import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import { MedicationEntry } from '~/lib/types/medications';

export default function MedicationList() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();
  const participantNumberId = Number.parseInt(participant_id);
  const [medications, setMedications] = useState<MedicationEntry[] | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchMedications = async () => {
      try {
        // wait 5 seconds
        //await new Promise((resolve) => setTimeout(resolve, 5000));
        const medicationsResult =
          await api.participants.fetchParticipantMedications(participantNumberId);
        setMedications(medicationsResult?.data);
        console.log({ medicationsResult });
        setError(undefined);
        setLoading(false);
      } catch (e: any) {
        setError(e.message as string);
        setLoading(false);
      }
    };
    fetchMedications();
  }, [participant_id]);

  return (
    <ProtectedRoute>
      <SafeAreaView style={styles.container} edges={['top']}>
        <ListMedications medications={medications} />
      </SafeAreaView>
    </ProtectedRoute>
  );
}
function ListMedications({ medications }: { medications: MedicationEntry[] | undefined }) {
  if (!medications) {
    return (
      <View style={styles.LoadingContainer}>
        <ActivityIndicator />
        <Text style={styles.LoadingMessage}>Loading...</Text>
      </View>
    );
  }
  if (medications.length === 0) {
    return (
      <NoDataScreen title="No Medications Found" subtitle="No Medications found for Participant" />
    );
  }
  return (
    <View style={styles.ListContainer}>
      <FlatList
        data={medications || []}
        renderItem={({ item }) => <MedicationItem medication={item} />}
        keyExtractor={(item) => item.id.toString()}
      />
    </View>
  );
}
function MedicationItem({ medication }: { medication: MedicationEntry }) {
  return (
    <View style={styles.MedicationItemContainer}>
      <Text style={styles.MedicationItemLabel}>
        {medication.name} - {medication.dosage}
      </Text>
      <Text>Frequency {medication.frequency}</Text>
      <DateOrUnknown date={medication.date_prescribed} name="Date Prescribed" />
      <DateOrUnknown date={medication.date_entered_into_system} name="Date Entered" />
    </View>
  );
}

function DateOrUnknown({ date, name }: { date?: string; name: string }) {
  if (!date) {
    return (
      <View style={styles.DateOrUnknownContainer}>
        <Ionicons name="alert-circle" size={24} color="red" />
        <Text style={styles.DateOrUnknownTextUnknown}>{name}: Unknown</Text>
      </View>
    );
  }
  return (
    <Text>
      {name}: {new Date(date).toLocaleDateString()}
    </Text>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    paddingTop: StatusBar.currentHeight,
  },
  scrollView: {},
  text: {
    fontSize: 42,
    padding: 12,
  },
  DateOrUnknownContainer: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  DateOrUnknownTextUnknown: {
    color: 'red',
  },
  MedicationItemContainer: {
    width: '50%',
    marginBottom: 16,
    borderWidth: 2,
    borderStyle: 'solid',
    borderColor: '#FFCDD2',
  },
  MedicationItemLabel: {
    fontSize: 24,
    color: 'black',
    fontWeight: 'bold',
  },
  LoadingContainer: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
  },
  LoadingMessage: {
    fontSize: 24,
    fontWeight: 'bold',
  },
  ListContainer: {
    marginLeft: 20,
    marginRight: 20,
  },
});
