import { Ionicons } from '@expo/vector-icons';
import { useLocalSearchParams } from 'expo-router';
import { useEffect, useState } from 'react';
import { ActivityIndicator, FlatList, StatusBar, StyleSheet, Text, View } from 'react-native';
import { SafeAreaView } from 'react-native-safe-area-context';
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
      <View className="flex flex-row items-center justify-center">
        <ActivityIndicator />
        <Text className="text-2xl font-bold">Loading...</Text>
      </View>
    );
  }
  if (medications.length === 0) {
    return (
      <View className="flex flex-row items-center justify-center">
        <Text className="text-2xl font-bold">No medications found</Text>;
      </View>
    );
  }
  return (
    <View className="mx-4">
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
    <View className="mb-4 border-2 border-solid border-red-100">
      <Text className="text-2xl font-bold">
        {medication.name} - {medication.dosage}
      </Text>
      <Text>Frequency {medication.frequency}</Text>
      <DateOrUnknown date={medication.date_prescribed} name="Date Prescribed" />
      <DateOrUnknown date={medication.date_entered_into_system} name="Date Entered" />
    </View>
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
});

function DateOrUnknown({ date, name }: { date?: string; name: string }) {
  if (!date) {
    return (
      <View className="flex flex-row items-center">
        <Ionicons name="alert-circle" size={24} color="red" />
        <Text className="color-red-600">{name}: Unknown</Text>
      </View>
    );
  }
  return (
    <Text>
      {name}: {new Date(date).toLocaleDateString()}
    </Text>
  );
}
