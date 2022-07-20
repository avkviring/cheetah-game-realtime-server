using System;
using System.Threading.Tasks;
using Cheetah.Matches.Relay.Shared;
using Cheetah.Matches.Relay.Shared.GRPC;

namespace Cheetah.Matches.Factory.Editor.Configurations
{
    /// <summary>
    /// Тестовая реализация сервиса получения имен шаблонов и полей
    /// </summary>
    public class TestConfigurationsProvider : ConfigurationsProvider
    {
        private readonly Random random = new Random();
        private readonly string[] templates = new[] { "tank", "user", "mine", "isida", null };
        private readonly string[] fields = new[] { "tank", "user", "mine", "isida", null };


        public string GetTemplateName(ushort template)
        {
            return templates[(int)(random.NextDouble() * templates.Length)];
        }

        public string GetFieldName(ushort id, FieldType type)
        {
            return fields[(int)(random.NextDouble() * fields.Length)];
        }

        public Task Load()
        {
            return Task.CompletedTask;
        }

        public Task Destroy()
        {
            return Task.CompletedTask;
        }
    }
}