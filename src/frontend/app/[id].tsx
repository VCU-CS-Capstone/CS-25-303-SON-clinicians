import { useRouter, useLocalSearchParams } from 'expo-router';
import React, { useState, useEffect } from 'react';
import { View, Text } from 'react-native';

import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';

export default function PatientInfo() {
  const { id } = useLocalSearchParams();
  const [patientInfo, setPatientInfo] = useState(null);

  useEffect(() => {
    const fetchPatientInfo = async () => {
      try {
        const patientInfo = await api.participants.fetchById(id[0]);
        setPatientInfo(patientInfo);
      } catch (error) {
        console.error(error);
      }
    };

    if (id) {
      fetchPatientInfo();
    }
  }, [id]);

  return (
    <ProtectedRoute>
      <View>
        {patientInfo ? (
          <Text>{JSON.stringify(patientInfo, null, 2)}</Text>
        ) : (
          <Text>Loading...</Text>
        )}
      </View>
    </ProtectedRoute>
  );
}
