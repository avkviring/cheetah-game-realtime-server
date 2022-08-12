using System;
using System.IO;
using System.Text;
using System.Xml;
using UnityEditor.Android;

namespace Cheetah.Android.Editor
{
    public class GooglePlayGamesBuild : IPostGenerateGradleAndroidProject
    {
        private string _manifestFilePath;

        public int callbackOrder { get; }

        public void OnPostGenerateGradleAndroidProject(string path)
        {
            var manifest = new AndroidXmlDocument(GetManifestPath(path));
            manifest.AddMeta("com.google.android.gms.games.APP_ID", "@string/app_id");
            manifest.AddMeta("com.google.android.gms.games.unityVersion", "0.10.12");
            manifest.AddMeta("com.google.games.bridge.NativeBridgeActivity", "@android:style/Theme.Translucent.NoTitleBar.Fullscreen");
            manifest.Save();

            var pathBuilder = new StringBuilder(path);
            pathBuilder.Append(Path.DirectorySeparatorChar).Append("src");
            pathBuilder.Append(Path.DirectorySeparatorChar).Append("main");
            pathBuilder.Append(Path.DirectorySeparatorChar).Append("res");
            pathBuilder.Append(Path.DirectorySeparatorChar).Append("values");
            pathBuilder.Append(Path.DirectorySeparatorChar).Append("strings.xml");

            var appId = GooglePlayGamesSettings.GetOrCreateSettings().AppId;
            if (appId.Length < 3)
            {
                throw new Exception("Please set GooglePlayGames APP_ID in " + GooglePlayGamesSettingsProvider.PATH);
            }

            var content = new StringBuilder();
            content.Append("<resources>\n");
            content.Append("<string name=\"app_id\">" + appId + "</string>\n");
            content.Append("</resources>");
            File.WriteAllText(pathBuilder.ToString(), content.ToString());
        }

        private string GetManifestPath(string basePath)
        {
            if (string.IsNullOrEmpty(_manifestFilePath))
            {
                var pathBuilder = new StringBuilder(basePath);
                pathBuilder.Append(Path.DirectorySeparatorChar).Append("src");
                pathBuilder.Append(Path.DirectorySeparatorChar).Append("main");
                pathBuilder.Append(Path.DirectorySeparatorChar).Append("AndroidManifest.xml");
                _manifestFilePath = pathBuilder.ToString();
            }

            return _manifestFilePath;
        }
    }

    internal class AndroidXmlDocument : XmlDocument
    {
        private string m_Path;
        protected XmlNamespaceManager nsMgr;
        public readonly string AndroidXmlNamespace = "http://schemas.android.com/apk/res/android";

        public AndroidXmlDocument(string path)
        {
            m_Path = path;
            using (var reader = new XmlTextReader(m_Path))
            {
                reader.Read();
                Load(reader);
            }

            nsMgr = new XmlNamespaceManager(NameTable);
            nsMgr.AddNamespace("android", AndroidXmlNamespace);
        }

        public string Save()
        {
            return SaveAs(m_Path);
        }

        public string SaveAs(string path)
        {
            using (var writer = new XmlTextWriter(path, new UTF8Encoding(false)))
            {
                writer.Formatting = Formatting.Indented;
                Save(writer);
            }

            return path;
        }

        public void AddMeta(string name, string value)
        {
            var node = SelectSingleNode("/manifest/application");
            XmlElement child = CreateElement("meta-data");
            node.AppendChild(child);
            child.Attributes.Append(CreateAndroidAttribute("name", name));
            child.Attributes.Append(CreateAndroidAttribute("value", value));
        }

        private XmlAttribute CreateAndroidAttribute(string key, string value)
        {
            XmlAttribute attr = CreateAttribute("android", key, AndroidXmlNamespace);
            attr.Value = value;
            return attr;
        }
    }
}