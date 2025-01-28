import { useLocalSearchParams, useRouter } from 'expo-router';
import React, { useState, useEffect } from 'react';
import { View, Text } from 'react-native';
import { Button } from '~/components/Button';
import LabelAndItem from '~/components/LabelAndItem';
import RecentVisits from '~/components/participant/RecentVisits';

import ProtectedRoute from '~/components/ProtectedRoute';
import api from '~/lib/api';
import { Participant } from '~/lib/types/participant';

export default function PatientInfo() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();

  return (
    <ProtectedRoute>
      <RecentVisits participantId={Number.parseInt(participant_id)} />
    </ProtectedRoute>
  );
}
