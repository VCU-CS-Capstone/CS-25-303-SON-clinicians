import { Stack } from 'expo-router';
import { Text, View } from 'react-native';

import { SafeAreaView } from 'react-native';
import api, { API_URL } from '~/lib/api';
import { useEffect, useState } from 'react';
import { SiteInfo } from '~/lib/RequestTypes';

export default function Home() {
  const [siteInfo, setSiteInfo] = useState<SiteInfo | undefined>(undefined);
  const [error, setError] = useState<string | undefined>(undefined);
  const [loading, setLoading] = useState(true);

  const fetchSiteInfo = async () => {
    try {
      const siteInfoResponse = await api.siteInfo();

      setSiteInfo(siteInfoResponse);
      setError(undefined);
      setLoading(false);
    } catch (e: any) {
      setError(e as string);
      setLoading(false);
    }
  };
  useEffect(() => {
    fetchSiteInfo();
  }, []);
  return (
    <>
      <Stack.Screen options={{ title: 'Home' }} />
      <SafeAreaView>
        <View style={{ padding: 20 }}>
          <Text style={styles.TextItem}>Hello World</Text>
          <Text style={styles.TextItem}>
            Connected to <Text style={styles.Link}>{API_URL}</Text>
          </Text>
          <SiteInfoComponent siteInfo={siteInfo} error={error} />
        </View>
      </SafeAreaView>
    </>
  );
}
function SiteInfoComponent({
  siteInfo,
  error,
}: {
  siteInfo: SiteInfo | undefined;
  error: string | undefined;
}) {
  if (!siteInfo && !error) {
    return null;
  }
  if (!siteInfo) {
    if (!error) {
      return null;
    }
    return <Text style={styles.Error}>{error}</Text>;
  }
  return (
    <View style={styles.BackendInfo}>
      <Text>Current Version {siteInfo.version}</Text>
      <Text>Backend Build Time {new Date(siteInfo.build_time).toLocaleString()}</Text>
    </View>
  );
}
const styles = {
  TextItem: {
    fontSize: 36,
  },
  Link: {
    color: 'blue',
  },
  Error: {
    color: 'red',
    FontSize: 36,
  },
  BackendInfo: {
    padding: 20,
    BorderWidth: 5,
    BorderColor: 'grey',

    MarginTop: 20,
  },
};
