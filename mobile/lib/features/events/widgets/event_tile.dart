import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../../../theme/app_colors.dart';

class EventTile extends StatelessWidget {
  const EventTile({
    super.key,
    required this.event,
    required this.timestamp,
    this.batteryPercentage,
  });

  final String event;
  final DateTime? timestamp;
  final double? batteryPercentage;

  bool get _accent =>
      event.contains('disconnect') ||
      event.contains('critical') ||
      event.contains('low');

  @override
  Widget build(BuildContext context) {
    final when = timestamp == null
        ? '—'
        : DateFormat('d MMM, HH:mm:ss').format(timestamp!.toLocal());
    final label = event.replaceAll('_', ' ');

    return Container(
      margin: const EdgeInsets.only(bottom: 10),
      padding: const EdgeInsets.fromLTRB(12, 12, 14, 12),
      decoration: BoxDecoration(
        border: Border(
          left: BorderSide(
            color: _accent ? AppColors.primary : AppColors.border,
            width: 3,
          ),
        ),
      ),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  label,
                  style: const TextStyle(fontWeight: FontWeight.w600),
                ),
                const SizedBox(height: 2),
                Text(
                  when,
                  style: const TextStyle(
                    color: AppColors.mutedForeground,
                    fontSize: 13,
                  ),
                ),
              ],
            ),
          ),
          if (batteryPercentage != null)
            Text(
              '${batteryPercentage!.round()}%',
              style: const TextStyle(fontWeight: FontWeight.w600),
            ),
        ],
      ),
    );
  }
}
