export const formatDate = (iso?: string): string => {
  if (!iso) return '';
  const d = new Date(iso);
  const dd = String(d.getDate()).padStart(2, '0');
  const mm = String(d.getMonth() + 1).padStart(2, '0');
  const yyyy = d.getFullYear();
  return `${dd}.${mm}.${yyyy}`;
};

export const formatTime = (iso?: string): string => {
  if (!iso) return '';
  const d = new Date(iso);
  const hh = String(d.getHours()).padStart(2, '0');
  const mi = String(d.getMinutes()).padStart(2, '0');
  return `${hh}:${mi}`;
};

export const formatDateTime = (iso?: string): string => {
  if (!iso) return '';
  return `${formatDate(iso)} ${formatTime(iso)}`;
};

export const pad2 = (n: number) => String(n).padStart(2, '0');

