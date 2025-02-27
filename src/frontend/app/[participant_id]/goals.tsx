import { useLocalSearchParams } from 'expo-router';
import { useEffect, useState } from 'react';
import { FlatList, StyleSheet, Text, View } from 'react-native';
import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import { Goal, GoalStep } from '~/lib/types/goals';

export default function GoalsList() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();
  const participantNumberId = Number.parseInt(participant_id);
  const [goals, setGoals] = useState<Goal[] | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchGoals = async () => {
      try {
        const goalsResponse = await api.participants.fetchGoalsForParticipant(participantNumberId);
        setGoals(goalsResponse);
        setError(undefined);
        setLoading(false);
      } catch (e: any) {
        setError(e.message as string);
        setLoading(false);
      }
    };
    fetchGoals();
  }, [participant_id]);

  return (

    <ProtectedRoute>
      {error ? <Text>{error}</Text> : null}
      <DisplayGoals goals={goals} />
    </ProtectedRoute>
  );
}
function DisplayGoals({ goals }: { goals: Goal[] | undefined }) {
  if (!goals) {
    return null;
  }
  if (goals.length === 0) {
    return (
      <View style={styles.goalContainer}>
        <Text style={styles.goalText}>No goals found</Text>
      </View>
    );
  }

  return (
    <FlatList
      data={goals}
      renderItem={({ item }) => <GoalView goal={item} />}
      keyExtractor={(item) => item.id.toString()}
    />
  );
}
const styles = StyleSheet.create({
  goalContainer:{
    marginBottom: 16,

  },
  goalText:{
    fontSize: 20,
    padding: 12,
  }
});
function GoalView({ goal }: { goal: Goal }) {
  const [steps, setSteps] = useState<GoalStep[] | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchSteps = async () => {
      try {
        const goalsResponse = await api.participants.fetchStepsForGoal(goal.id);
        setSteps(goalsResponse);
        setError(undefined);
        setLoading(false);
      } catch (e: any) {
        setError(e.message as string);
        setLoading(false);
      }
    };
    fetchSteps();
  }, [goal]);
  return (
    <View style={styles.goalContainer}>
      <Text style={styles.goalText}>{goal.goal}</Text>;
      <GoalSteps steps={steps} />
    </View>
  );
}
const stepStyles = StyleSheet.create({
  stepContainer:{
    marginBottom: 16,
    borderWidth: 2,
    borderStyle: 'solid',
    borderColor: '#FFCCCC',
  },
  stepText:{
    fontSize: 20,
    padding: 12,
  }
});

function GoalSteps({ steps }: { steps: GoalStep[] | undefined }) {
  if (!steps) {
    return null;
  }
  if (steps.length === 0) {
    return (
      <View style={stepStyles.stepContainer}>
        <Text style={stepStyles.stepText}>No steps found</Text>;
      </View>)
  }
  return (
    <View>
      <Text style={
        {
          fontSize: 20,
          padding: 12,
          fontWeight: 'bold',
        }
      }>Steps</Text>
      <FlatList
        data={steps}
        renderItem={({ item }) => <StepView step={item} />}
        keyExtractor={(item) => item.id.toString()}
      />
    </View>
  );
}

function StepView({ step }: { step: GoalStep }) {
  return (
    <View style={stepStyles.stepContainer}>
      <Text style={stepStyles.stepText}>{step.step}</Text>
    </View >
  );
}
