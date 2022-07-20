using UnityEditor;

namespace Cheetah.Platform.Editor.Connector
{
    public static class LocalClusterConnectorFactory
    {
        private const string PrefsKey = "cheetah_platform_admin_grpc_connector";
        private const string PrefsHostKey = PrefsKey + "_host";
        private const string PrefsPortKey = PrefsKey + "_port";

        public static void Configure(string host, int port)
        {
            EditorPrefs.SetString(PrefsHostKey, host);
            EditorPrefs.SetInt(PrefsPortKey, port);
        }

        public static ClusterConnector CreateConnector()
        {
            var host = EditorPrefs.GetString(PrefsHostKey, "localhost");
            var port = EditorPrefs.GetInt(PrefsPortKey, 7777);
            return new ClusterConnector(host, port, false);
        }
    }
}