using System.Threading.Tasks;
using Cheetah.Matches.Realtime.Shared.GRPC;
using JetBrains.Annotations;

namespace Cheetah.Matches.Factory.Editor.Configurations
{
    /// <summary>
    /// Интерфейс для получения имен шаблонов и полей
    /// </summary>
    public interface ConfigurationsProvider
    {
        [CanBeNull]
        string GetTemplateName(ushort template);

        [CanBeNull]
        string GetFieldName(ushort id, FieldType type);

        Task Load();
        Task Destroy();
    }
}