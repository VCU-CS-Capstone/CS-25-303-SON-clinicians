import { Stack, useLocalSearchParams } from 'expo-router';
import { useEffect, useState } from 'react';
import { Dimensions, ScrollView, StatusBar, StyleSheet, Text, View } from 'react-native';
import { FlatList } from 'react-native-gesture-handler';
import { BarChart, LineChart, lineDataItem } from 'react-native-gifted-charts';
import { SafeAreaView } from 'react-native-safe-area-context';
import { NoDataScreen } from '~/components/NoDataScreen';
import {
  BloodPressureSelector,
  BloodPressureType,
  GraphOrDataSelector,
  GraphOrDataValue,
} from '~/components/participant/bloodPressureTrends/BloodPressureTrendSelector';
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
  const [currentReading, setCurrentReading] = useState<BloodPressureType>(BloodPressureType.Sit);
  const [graphOrData, setGraphOrData] = useState<GraphOrDataValue>(GraphOrDataValue.Graph);
  const [sitTrends, setSitTrends] = useState<BloodPressureStatsOneReading[] | undefined>(undefined);
  const [standTrends, setStandTrends] = useState<BloodPressureStatsOneReading[] | undefined>(
    undefined
  );
  const [personalTrends, setPersonalTrends] = useState<BloodPressureStatsOneReading[] | undefined>(
    undefined
  );
  useEffect(() => {
    if (!trends) {
      setSitTrends(undefined);
      return;
    }
    const sitTrends = [];
    const standTrends = [];
    const personalTrends = [];
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
      if (trend.readings.personal) {
        personalTrends.push({
          case_note_id: trend.case_note_id,
          date_of_visit: new Date(trend.date_of_visit),
          blood_pressure: trend.readings.personal,
        });
      }
    }
    setSitTrends(sitTrends);
    setStandTrends(standTrends);
    setPersonalTrends(personalTrends);
  }, [trends]);
  if (!trends || trends.length === 0) {
    return (
      <NoDataScreen
        title="No Blood Pressure Readings"
        subtitle="No Blood Pressure Readings Present"
      />
    );
  }
  return (
    <View>
      <View style={{ flexDirection: 'row' }}>
        <BloodPressureSelector onChange={setCurrentReading} />
        <GraphOrDataSelector onChange={setGraphOrData} />
      </View>
      <TrendsView
        trends={sitTrends}
        activeType={currentReading}
        type={BloodPressureType.Sit}
        dataOrGraph={graphOrData}
      />
      <TrendsView
        trends={standTrends}
        activeType={currentReading}
        type={BloodPressureType.Standing}
        dataOrGraph={graphOrData}
      />
      <TrendsView
        trends={personalTrends}
        activeType={currentReading}
        type={BloodPressureType.Personal}
        dataOrGraph={graphOrData}
      />
    </View>
  );
}
function TrendsView({
  trends,
  activeType,
  type,
  dataOrGraph,
}: {
  trends: BloodPressureStatsOneReading[] | undefined;
  activeType: BloodPressureType;
  type: BloodPressureType;
  dataOrGraph: GraphOrDataValue;
}) {
  if (activeType !== type) {
    return null;
  }
  if (!trends || trends.length === 0) {
    return (
      <NoDataScreen
        title="No Blood Pressure Readings"
        subtitle="No Blood Pressure Readings Present"
      />
    );
  }
  if (dataOrGraph === GraphOrDataValue.Graph) {
    return <BpLineChart trends={trends} />;
  }
  return (
    <View>
      <ListTrends trends={trends} />
    </View>
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

function ListTrends({ trends }: { trends: BloodPressureStatsOneReading[] | undefined }) {
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

function ListTrendItem({ trend }: { trend: BloodPressureStatsOneReading }) {
  return (
    <View style={{ flexDirection: 'row', marginVertical: 8, borderBottomWidth: 2 }}>
      <Text
        style={{
          fontSize: 18,
          fontWeight: 'bold',
          paddingLeft: 8,
        }}
      >
        {trend.date_of_visit.toLocaleDateString()}
      </Text>
      <Text
        style={{
          fontSize: 18,
          marginLeft: 8,
        }}
      >
        {trend.blood_pressure.systolic} / {trend.blood_pressure.diastolic}
      </Text>
    </View>
  );
}
