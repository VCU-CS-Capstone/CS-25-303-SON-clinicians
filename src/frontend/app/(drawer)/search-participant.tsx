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
      <View className="flex flex-row">
        <TextInput
          className="w-1/4"
          style={styles.input}
          value={firstName}
          onChangeText={setFirstName}
          placeholder="First name"
        ></TextInput>
        <TextInput
          className="w-1/4"
          style={styles.input}
          value={lastName}
          onChangeText={setLastName}
          placeholder="Last name"
        ></TextInput>
        <View className="flex-1">
          <ProgramSelector
            onChange={(value) => {
              setProgramValue(value);
            }}
            allowNone={true}
          />
        </View>
      </View>
      <SafeAreaView style={{ flex: 1 }}>
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
    margin: 12,
    borderWidth: 1,
    padding: 10,
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
      <View className="mb-4 w-1/4 border-2 border-solid border-red-100">
        <Text className="text-xl font-bold">
          {participant.first_name} {participant.last_name}
        </Text>
        <Text>{participant.program}</Text>
        <LocationName locationId={participant.location} />
      </View>
    </Link>
  );
}
