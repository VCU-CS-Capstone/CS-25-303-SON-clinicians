import { useLocalSearchParams } from 'expo-router';
import { useEffect, useState } from 'react';
import { Dimensions, ScrollView, StatusBar, StyleSheet, Text, View } from 'react-native';
import { FlatList } from 'react-native-gesture-handler';
import { BarChart, LineChart, lineDataItem } from 'react-native-gifted-charts';
import { SafeAreaView } from 'react-native-safe-area-context';
import { NoDataScreen } from '~/components/NoDataScreen';
import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import { GlucoseEntry } from '~/lib/types/stats';

export default function PatientInfo() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();
  const participantNumberId = Number.parseInt(participant_id);
  const [trends, setTrends] = useState<GlucoseEntry[] | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchPatient = async () => {
    try {
      const trendsResponse = await api.participants.fetchGlucoseHistory(participantNumberId);
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
function ShowTrends({ trends }: { trends: GlucoseEntry[] | undefined }) {
  if (!trends || trends.length === 0) {
    return <NoDataScreen title="No Glucose Readings" subtitle="No Glucose Readings Present" />;
  }
  return (
    <View>
      <GlucoseLineChart trends={trends} />
      <ListTrends trends={trends} />
    </View>
  );
}
function ListTrends({ trends }: { trends: GlucoseEntry[] | undefined }) {
  return (
    <FlatList
      data={trends}
      renderItem={({ item }) => <ListTrendItem trend={item} />}
      keyExtractor={(item) => item.case_note_id.toString()}
    />
  );
}
function ListTrendItem({ trend }: { trend: GlucoseEntry }) {
  return (
    <View>
      <Text>{trend.date_of_visit}</Text>
      <Text>
        {trend.result} - {trend.fasting ? 'Fasting' : 'Non-fasting'}
      </Text>
    </View>
  );
}

function GlucoseLineChart({ trends }: { trends: GlucoseEntry[] | undefined }) {
  const [fastingValues, setfastingValues] = useState<lineDataItem[]>([]);
  const [nonFastingValues, setNonFastingValues] = useState<lineDataItem[]>([]);
  useEffect(() => {
    if (!trends) {
      return;
    }
    const fasting: lineDataItem[] = [];
    const nonFasting: lineDataItem[] = [];
    for (const trend of trends) {
      if (trend.fasting) {
        fasting.push({ value: trend.result, label: trend.date_of_visit });
      } else {
        nonFasting.push({ value: trend.result, label: trend.date_of_visit });
      }
    }
    setfastingValues(fasting);
    setNonFastingValues(nonFasting);
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
        data={fastingValues}
        data2={nonFastingValues}
        spacing={100}
        showVerticalLines
      />
    </View>
  );
}
