import { useLocalSearchParams } from 'expo-router';
import { useEffect, useState } from 'react';
import { Dimensions, ScrollView, StatusBar, StyleSheet, Text, View } from 'react-native';
import { FlatList } from 'react-native-gesture-handler';
import { BarChart, LineChart, lineDataItem } from 'react-native-gifted-charts';
import { SafeAreaView } from 'react-native-safe-area-context';
import { NoDataScreen } from '~/components/NoDataScreen';
import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import { WeightEntry } from '~/lib/types/stats';

export default function PatientInfo() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();
  const participantNumberId = Number.parseInt(participant_id);
  const [trends, setTrends] = useState<WeightEntry[] | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchPatient = async () => {
    try {
      const trendsResponse = await api.participants.fetchWeightHistory(participantNumberId);
      if (trendsResponse) {
        setTrends(trendsResponse.data);
      }
      setError(undefined);
      setLoading(false);
    } catch (e: any) {
      setError(e.message as string);
      setLoading(false);
    }
  };
  useEffect(() => {
    fetchPatient();
  }, []);

  return (
    <ProtectedRoute>
      <SafeAreaView style={styles.container} edges={['top']}>
        <ShowTrends trends={trends} />
      </SafeAreaView>
    </ProtectedRoute>
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
function ShowTrends({ trends }: { trends: WeightEntry[] | undefined }) {
  if (!trends || trends.length === 0) {
    return <NoDataScreen title="No Weight Data" subtitle="No weight data found for participant" />;
  }
  return (
    <View>
      <WeightLineChart trends={trends} />
      <ListTrends trends={trends} />
    </View>
  );
}
function ListTrends({ trends }: { trends: WeightEntry[] | undefined }) {
  if (!trends) {
    return null;
  }
  return (
    <FlatList
      data={trends}
      renderItem={({ item }) => <ListTrendItem trend={item} />}
      keyExtractor={(item) => item.case_note_id.toString()}
    />
  );
}
function ListTrendItem({ trend }: { trend: WeightEntry }) {
  return (
    <View>
      <Text>{trend.date_of_visit}</Text>
      <Text>{trend.weight}</Text>
    </View>
  );
}

function WeightLineChart({ trends }: { trends: WeightEntry[] | undefined }) {
  const [weights, setWeights] = useState<lineDataItem[]>([]);

  useEffect(() => {
    if (!trends) {
      return;
    }
    const weights: lineDataItem[] = [];
    for (const trend of trends) {
      weights.push({
        value: trend.weight,
      });
    }
    setWeights(weights);
  }, [trends]);
  if (!trends) {
    return null;
  }
  const windowWidth = Dimensions.get('window').width - 100;
  return (
    <View>
      <LineChart
        initialSpacing={0}
        width={windowWidth}
        thickness1={5}
        color1="blue"
        thickness2={5}
        color2="red"
        data={weights}
        spacing={100}
        showVerticalLines
      />
    </View>
  );
}
