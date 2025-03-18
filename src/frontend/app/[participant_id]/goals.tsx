import { Ionicons } from '@expo/vector-icons';
import { useLocalSearchParams } from 'expo-router';
import { useEffect, useState } from 'react';
import { FlatList, StyleSheet, Text, View } from 'react-native';
import { NoDataScreen } from '~/components/NoDataScreen';
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
      <NoDataScreen
        title="No Goals Present"
        subtitle="No goals have been set for this participant"
      />
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
  goalContainer: {
    marginBottom: 16,
    borderWidth: 2,
  },
  goalText: {
    fontSize: 20,
    padding: 12,
  },
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
      <Text style={styles.goalText}>{goal.goal}</Text>
      <GoalSteps steps={steps} />
    </View>
  );
}
const stepStyles = StyleSheet.create({
  stepContainer: {
    marginBottom: 16,
    borderWidth: 2,
    borderStyle: 'solid',
    borderColor: '#FFCCCC',
    marginLeft: 12,
    marginRight: 6,
  },
  stepText: {
    fontSize: 20,
    padding: 12,
  },
  stepTextBox: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  stepDetailsText: {
    fontSize: 16,
    padding: 6,
  },
  DateOrUnknownContainer: {
    flexDirection: 'row',
    alignItems: 'center',
  },
  DateOrUnknownTextUnknown: {
    color: 'red',
  },
});

function GoalSteps({ steps }: { steps: GoalStep[] | undefined }) {
  if (!steps) {
    return null;
  }
  if (steps.length === 0) {
    return (
      <View style={stepStyles.stepContainer}>
        <Text style={stepStyles.stepText}>No steps found</Text>;
      </View>
    );
  }
  return (
    <View>
      <Text
        style={{
          fontSize: 20,
          padding: 12,
          fontWeight: 'bold',
        }}
      >
        Steps
      </Text>
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
      <View style={stepStyles.stepTextBox}>
        <CompletedOrNot completed={step.is_complete} />
        <Text style={stepStyles.stepText}>{step.step}</Text>
      </View>
      <DateOrUnknown date={step.date_set} name="Date Set" />
      <DateOrUnknown date={step.date_to_be_completed} name="Date Due" />
    </View>
  );
}
function CompletedOrNot({ completed }: { completed?: boolean }) {
  if (completed) {
    return <Ionicons name="checkbox-outline" size={24} color="green" />;
  }
  return <Ionicons name="close-circle-outline" size={24} color="red" />;
}
function DateOrUnknown({ date, name }: { date?: string; name: string }) {
  if (!date) {
    return (
      <View style={stepStyles.DateOrUnknownContainer}>
        <Ionicons name="alert-circle" size={24} color="red" />
        <Text style={stepStyles.stepDetailsText}>{name}: Unknown</Text>
      </View>
    );
  }
  return (
    <Text style={stepStyles.stepDetailsText}>
      {name}: {new Date(date).toLocaleDateString()}
    </Text>
  );
}
