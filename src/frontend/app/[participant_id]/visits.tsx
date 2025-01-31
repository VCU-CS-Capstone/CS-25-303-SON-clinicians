import { useLocalSearchParams } from 'expo-router';
import React, { useState, useEffect } from 'react';
import RecentVisits from '~/components/participant/RecentVisits';

import ProtectedRoute from '~/components/ProtectedRoute';

export default function PatientInfo() {
  const { participant_id } = useLocalSearchParams<{ participant_id: string }>();

  return (
    <ProtectedRoute>
      <RecentVisits participantId={Number.parseInt(participant_id)} />
    </ProtectedRoute>
  );
}
