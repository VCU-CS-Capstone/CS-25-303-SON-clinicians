import { Stack, useLocalSearchParams } from 'expo-router';
import { useEffect, useState } from 'react';
import { Dimensions, ScrollView, StatusBar, StyleSheet, Text, View } from 'react-native';
import { FlatList } from 'react-native-gesture-handler';
import { BarChart, LineChart, lineDataItem } from 'react-native-gifted-charts';
import { SafeAreaView } from 'react-native-safe-area-context';
import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import {
  BloodPressureReading,
  BloodPressureStats,
  BloodPressureStatsOneReading,
} from '~/lib/types/stats';

export default function PatientInfo() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();
  const participantNumberId = Number.parseInt(participant_id);
  const [trends, setTrends] = useState<BloodPressureStats[] | undefined>(undefined);
  const [sitTrends, setSitTrends] = useState<BloodPressureStatsOneReading[] | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);
  const fetchPatient = async () => {
    try {
      const trendsResponse = await api.participants.fetchBpHistory(participantNumberId);
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
    <>
      <Stack.Screen options={{ title: 'Blood Pressure' }} />
      <ProtectedRoute>
        <SafeAreaView style={styles.container} edges={['top']}>
          <ShowTrends trends={trends} />
        </SafeAreaView>
      </ProtectedRoute>
    </>
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
function ShowTrends({ trends }: { trends: BloodPressureStats[] | undefined }) {
  const [sitTrends, setSitTrends] = useState<BloodPressureStatsOneReading[] | undefined>(undefined);
  const [standTrends, setStandTrends] = useState<BloodPressureStatsOneReading[] | undefined>(
    undefined
  );
  useEffect(() => {
    if (!trends) {
      setSitTrends(undefined);
      return;
    }
    const sitTrends = [];
    const standTrends = [];
    for (const trend of trends) {
      if (trend.readings.sit) {
        sitTrends.push({
          case_note_id: trend.case_note_id,
          date_of_visit: new Date(trend.date_of_visit),
          blood_pressure: trend.readings.sit,
        });
      }
      if (trend.readings.stand) {
        standTrends.push({
          case_note_id: trend.case_note_id,
          date_of_visit: new Date(trend.date_of_visit),
          blood_pressure: trend.readings.stand,
        });
      }
    }
    setSitTrends(sitTrends);
    setStandTrends(standTrends);
  }, [trends]);
  if (!trends) {
    return null;
  }

  return (
    <View>
      <BpLineChart trends={sitTrends} />
      <BpLineChart trends={standTrends} />
      <ListTrends trends={trends} />
    </View>
  );
}
function ListTrends({ trends }: { trends: BloodPressureStats[] | undefined }) {
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
function ListTrendItem({ trend }: { trend: BloodPressureStats }) {
  return (
    <View className="mb-4 border-2 border-solid border-red-100">
      <Text className="text-xl">Visit {new Date(trend.date_of_visit).toLocaleDateString()}</Text>
      <Reading type="Sit" trend={trend.readings.sit} />
      <Reading type="Standing" trend={trend.readings.stand} />
      <Reading type="Personal" trend={trend.readings.personal} />
    </View>
  );
}
function Reading({ type, trend }: { type: string; trend: BloodPressureReading | undefined }) {
  if (!trend) {
    return null;
  }
  return (
    <Text>
      {type} -{trend.systolic}/{trend.diastolic}
    </Text>
  );
}

function BpLineChart({ trends }: { trends: BloodPressureStatsOneReading[] | undefined }) {
  const [sys, setSys] = useState<lineDataItem[]>([]);
  const [dia, setDia] = useState<lineDataItem[]>([]);

  useEffect(() => {
    if (!trends) {
      return;
    }
    const dataSys: lineDataItem[] = [];
    const dataDia: lineDataItem[] = [];
    for (const trend of trends) {
      dataSys.push({
        value: trend.blood_pressure.systolic,
        label: trend.date_of_visit.toLocaleDateString(),
      });
      dataDia.push({
        value: trend.blood_pressure.diastolic,
        label: trend.date_of_visit.toLocaleDateString(),
      });
    }
    setSys(dataSys);
    setDia(dataDia);
  }, [trends]);
  if (!trends) {
    return null;
  }
  const windowWidth = Dimensions.get('window').width - 100;
  return (
    <View>
      <LineChart
        maxValue={200}
        initialSpacing={0}
        width={windowWidth}
        thickness1={5}
        color1="blue"
        thickness2={5}
        color2="red"
        data={sys}
        data2={dia}
        spacing={100}
        showVerticalLines
      />
    </View>
  );
}
