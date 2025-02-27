import { AntDesign } from '@expo/vector-icons';
import { Link, router, useRouter } from 'expo-router';
import { useEffect, useState } from 'react';
import { FlatList, ScrollView, StyleSheet, Text, TextInput, View } from 'react-native';
import { Dropdown } from 'react-native-element-dropdown';
import { SafeAreaView } from 'react-native-safe-area-context';
import { LocationName } from '~/components/LocationName';
import { ProgramSelector } from '~/components/ProgramSelector';
import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import { Participant, ParticipantLookupResponse } from '~/lib/types/participant';

export default function SearchParticipant() {
  const [participants, setParticipants] = useState<ParticipantLookupResponse[] | undefined>(
    undefined
  );
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);

  const [programValue, setProgramValue] = useState<string | null>(null);
  const [firstName, setFirstName] = useState('');
  const [lastName, setLastName] = useState('');

  const lookupParticipants = async () => {
    try {
      const participantsResponse = await api.participants.lookup({
        program: programValue,
        first_name: firstName,
        last_name: lastName,
      });

      setParticipants(participantsResponse.data);
    } catch (e: any) {
      setError(e.message as string);
      setLoading(false);
    }
  };
  useEffect(() => {
    lookupParticipants();
  }, [programValue, firstName, lastName]);

  return (
    <ProtectedRoute>
      <View style={styles.row}>
        <TextInput
          style={[styles.input, styles.inputWidth]}
          value={firstName}
          onChangeText={setFirstName}
          placeholder="First name"
        />
        <TextInput
          style={[styles.input, styles.inputWidth]}
          value={lastName}
          onChangeText={setLastName}
          placeholder="Last name"
        />
        <View style={styles.inputWidth}>
          <ProgramSelector
            onChange={(value) => {
              setProgramValue(value);
            }}
            allowNone={true}
          />
        </View>
      </View>
      <SafeAreaView style={styles.flexOne}>
        <FlatList
          data={participants}
          renderItem={({ item }) => <ParticipantLookupItem participant={item} />}
          keyExtractor={(item) => item.id.toString()}
        />
      </SafeAreaView>
    </ProtectedRoute>
  );
}

const styles = StyleSheet.create({
  input: {
    height: 40,
    marginVertical: 8,
    borderWidth: 1,
    borderRadius: 8,
    padding: 10,
  },
  flexOne: {
    flex: 1,
  },
  row: {
    flexDirection: 'row',
    alignItems: 'center', // Ensures elements are aligned vertically
    justifyContent: 'space-between', // Keeps spacing even
    paddingHorizontal: 12, // Adjusts spacing
  },
  inputWidth: {
    width: '30%', // Ensures all inputs and dropdown are the same width
  },
  LookupItem: {
    marginBottom: 4,
    borderWidth: 2,
    padding: 4,
    width: '25%',
  },
  LookupItemLabel: {
    fontWeight: 'bold',
    fontSize: 18,
  },
});

function ParticipantLookupItem({ participant }: { participant: ParticipantLookupResponse }) {
  const router = useRouter();

  return (
    <Link
      href={{
        pathname: `/[participant_id]/overview`,
        params: { participant_id: participant.id },
      }}
    >
      <View style={styles.LookupItem}>
        <Text style={styles.LookupItemLabel}>
          {participant.first_name} {participant.last_name}
        </Text>
        <Text>{participant.program}</Text>
        <LocationName locationId={participant.location} />
      </View>
    </Link>
  );
}
