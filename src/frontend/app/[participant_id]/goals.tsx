import { useLocalSearchParams } from 'expo-router';
import { useEffect, useState } from 'react';
import { Text } from 'react-native';
import ProtectedRoute from '~/components/ProtectedRoute';
import { Goal } from '~/lib/types/goals';

export default function GoalsList() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();
  const participantNumberId = Number.parseInt(participant_id);
  const [goals, setGoals] = useState<Goal[] | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchGoals = async () => {};
    fetchGoals();
  }, [participant_id]);

  return (
    <ProtectedRoute>
      <Text>Hello world</Text>
    </ProtectedRoute>
  );
}
