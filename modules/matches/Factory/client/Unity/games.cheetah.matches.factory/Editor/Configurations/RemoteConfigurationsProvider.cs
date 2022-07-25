using System.Collections.Generic;
using System.Threading.Tasks;
using Cheetah.Matches.Factory.Editor.GRPC;
using Cheetah.Matches.Realtime.Shared.GRPC;
using Cheetah.Platform;
using JetBrains.Annotations;
using UnityEditor;

namespace Cheetah.Matches.Factory.Editor.Configurations
{
    public class RemoteConfigurationsProvider : ConfigurationsProvider
    {
        private ClusterConnector connector;
        private readonly Dictionary<ushort, string> templates = new Dictionary<ushort, string>();
        [CanBeNull] private Dictionary<FieldType, Dictionary<ushort, string>> fields = new Dictionary<FieldType, Dictionary<ushort, string>>();

        public RemoteConfigurationsProvider(ClusterConnector connector)
        {
            this.connector = connector;
            EditorApplication.quitting += ApplicationQuitting;
        }

        public Task Load()
        {
            return connector.DoRequest(async channel =>
            {
                var client = new GRPC.Configurations.ConfigurationsClient(channel);
                var response = await client.GetItemNamesAsync(new GetItemsNamesRequest());
                templates.Clear();
                fields.Clear();

                foreach (var template in response.Templates)
                {
                    templates[(ushort)template.Id] = template.Name;
                }

                foreach (var field in response.Fields)
                {
                    if (!fields.TryGetValue(field.Type, out var fieldsOfType))
                    {
                        fieldsOfType = new Dictionary<ushort, string>();
                        fields[field.Type] = fieldsOfType;
                    }

                    fieldsOfType[(ushort)field.Id] = field.Name;
                }
            });
        }


        public string GetTemplateName(ushort template)
        {
            return templates.TryGetValue(template, out var name) ? name : null;
        }

        public string GetFieldName(ushort id, FieldType type)
        {
            return fields.TryGetValue(type, out var names) && names.TryGetValue(id, out var name) ? name : null;
        }

        public async Task Destroy()
        {
            var tmpConnector = connector;
            connector = null;
            await tmpConnector.Destroy();
        }

        public async void ApplicationQuitting()
        {
            await Destroy();
        }
    }
}