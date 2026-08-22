import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../../../theme/app_colors.dart';
import '../../../widgets/glass_panel.dart';

class EventTile extends StatelessWidget {
  const EventTile({
    super.key,
    required this.event,
    required this.timestamp,
    this.batteryPercentage,
    this.accent,
  });

  final String event;
  final DateTime? timestamp;
  final double? batteryPercentage;
  final Color? accent;

  bool get _alert =>
      event.contains('disconnect') ||
      event.contains('critical') ||
      event.contains('low');

  @override
  Widget build(BuildContext context) {
    final when = timestamp == null
        ? '—'
        : DateFormat('d MMM, HH:mm:ss').format(timestamp!.toLocal());
    final label = event.replaceAll('_', ' ');
    final title = label.isEmpty
        ? 'Unknown'
        : label[0].toUpperCase() + label.substring(1);
    final lineColor = _alert ? (accent ?? AppColors.primary) : AppColors.stroke;

    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: GlassPanel(
        padding: const EdgeInsets.fromLTRB(14, 12, 14, 12),
        child: Row(
          children: [
            Container(
              width: 4,
              height: 40,
              decoration: BoxDecoration(
                color: lineColor,
                borderRadius: BorderRadius.circular(4),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    title,
                    style: const TextStyle(fontWeight: FontWeight.w700),
                  ),
                  const SizedBox(height: 2),
                  Text(
                    when,
                    style: const TextStyle(
                      color: AppColors.textDim,
                      fontSize: 13,
                    ),
                  ),
                ],
              ),
            ),
            if (batteryPercentage != null)
              Text(
                '${batteryPercentage!.round()}%',
                style: TextStyle(
                  fontWeight: FontWeight.w700,
                  color: accent ?? AppColors.text,
                ),
              ),
          ],
        ),
      ),
    );
  }
}
