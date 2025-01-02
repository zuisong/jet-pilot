const template = `apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: new-pvc
spec:
  accessModes:
    - ReadWriteOnce
  volumeMode: Filesystem
  resources:
    requests:
      storage: 8Gi
  storageClassName: slow
  selector:
    matchLabels:
      release: "stable"
    matchExpressions:
      - { key: environment, operator: In, values: [dev] }
`;

export default template;
